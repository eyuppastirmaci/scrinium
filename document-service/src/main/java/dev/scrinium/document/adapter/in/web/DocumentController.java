package dev.scrinium.document.adapter.in.web;

import dev.scrinium.document.adapter.in.web.config.UploadProperties;
import dev.scrinium.document.common.FormatUtils;
import dev.scrinium.document.domain.exception.TooManyFilesException;
import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentDownload;
import dev.scrinium.document.domain.model.DocumentMetadata;
import dev.scrinium.document.domain.model.DocumentPage;
import dev.scrinium.document.domain.model.DocumentThumbnail;
import dev.scrinium.document.domain.model.ExtractedPage;
import dev.scrinium.document.domain.port.in.DeleteDocument;
import dev.scrinium.document.domain.port.in.DownloadDocument;
import dev.scrinium.document.domain.port.in.GetDocument;
import dev.scrinium.document.domain.port.in.ListDocuments;
import dev.scrinium.document.domain.port.in.RetryProcessing;
import dev.scrinium.document.domain.port.in.UploadDocument;
import dev.scrinium.document.domain.port.out.DocumentStorage;
import dev.scrinium.document.domain.port.out.ProcessingResultRepository;
import org.springframework.core.io.InputStreamResource;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.multipart.MultipartFile;

import java.io.InputStream;
import java.time.OffsetDateTime;
import java.util.List;
import java.util.Set;
import java.util.UUID;

@RestController
@RequestMapping("/documents")
public class DocumentController {

    private final UploadDocument uploadDocument;
    private final ListDocuments listDocuments;
    private final GetDocument getDocument;
    private final DownloadDocument downloadDocument;
    private final DeleteDocument deleteDocument;
    private final RetryProcessing retryProcessing;
    private final UploadProperties uploadProperties;
    private final DocumentUploadRequestMapper uploadRequestMapper;
    private final ProcessingResultRepository processingResultRepository;
    private final DocumentStorage documentStorage;

    public DocumentController(UploadDocument uploadDocument,
                              ListDocuments listDocuments,
                              GetDocument getDocument,
                              DownloadDocument downloadDocument,
                              DeleteDocument deleteDocument,
                              RetryProcessing retryProcessing,
                              UploadProperties uploadProperties,
                              DocumentUploadRequestMapper uploadRequestMapper,
                              ProcessingResultRepository processingResultRepository,
                              DocumentStorage documentStorage) {
        this.uploadDocument = uploadDocument;
        this.listDocuments = listDocuments;
        this.getDocument = getDocument;
        this.downloadDocument = downloadDocument;
        this.deleteDocument = deleteDocument;
        this.retryProcessing = retryProcessing;
        this.uploadProperties = uploadProperties;
        this.uploadRequestMapper = uploadRequestMapper;
        this.processingResultRepository = processingResultRepository;
        this.documentStorage = documentStorage;
    }

    @GetMapping("/upload-constraints")
    public UploadConstraintsResponse uploadConstraints() {
        return new UploadConstraintsResponse(
                uploadProperties.supportedContentTypes(),
                uploadProperties.maxFileSize().toBytes(),
                FormatUtils.toMegabytes(uploadProperties.maxFileSize().toBytes()),
                uploadProperties.maxFilesPerRequest()
        );
    }

    @PostMapping(consumes = "multipart/form-data")
    public ResponseEntity<List<UploadResult>> upload(
            @RequestParam("file") List<MultipartFile> files
    ) {
        // Reject requests that exceed the configured file count limit.
        if (files.size() > uploadProperties.maxFilesPerRequest()) {
            throw new TooManyFilesException(files.size(), uploadProperties.maxFilesPerRequest());
        }

        // Process each file independently; collect results with per-file success or failure.
        List<UploadResult> results = files.stream()
                .map(this::uploadSingle)
                .toList();

        return ResponseEntity.accepted().body(results);
    }

    private UploadResult uploadSingle(MultipartFile file) {
        try {
            Document document = uploadDocument.upload(uploadRequestMapper.toCommand(file));
            return UploadResult.success(document.id(), document.status().name());
        } catch (RuntimeException e) {
            return UploadResult.failure(file.getOriginalFilename(), e.getMessage());
        }
    }

    @GetMapping
    public ListDocumentsResponse list(
            @RequestParam(defaultValue = "0") int page,
            @RequestParam(defaultValue = "20") int size
    ) {
        DocumentPage result = listDocuments.list(new ListDocuments.Query(page, size));

        List<DocumentSummary> items = result.items().stream()
                .map(d -> new DocumentSummary(
                        d.id(), d.fileName(), d.contentType(),
                        d.sizeBytes(), d.status().name(), d.createdAt()))
                .toList();

        return new ListDocumentsResponse(
                items, result.page(), result.size(),
                result.totalElements(), result.hasNext());
    }

    @GetMapping("/{id}")
    public DocumentDetailResponse detail(@PathVariable UUID id) {
        Document d = getDocument.get(id);
        return new DocumentDetailResponse(
                d.id(), d.fileName(), d.contentType(), d.sizeBytes(),
                d.sha256(), d.status().name(), d.failureReason(),
                d.createdAt(), d.updatedAt());
    }

    @GetMapping("/{id}/download")
    public ResponseEntity<InputStreamResource> download(@PathVariable UUID id) {
        DocumentDownload dl = downloadDocument.download(id);

        return ResponseEntity.ok()
                .header(HttpHeaders.CONTENT_DISPOSITION,
                        "attachment; filename=\"" + dl.fileName() + "\"")
                .contentType(MediaType.parseMediaType(dl.contentType()))
                .contentLength(dl.sizeBytes())
                .body(new InputStreamResource(dl.content()));
    }

    @DeleteMapping("/{id}")
    public ResponseEntity<Void> delete(@PathVariable UUID id) {
        deleteDocument.delete(id);
        return ResponseEntity.noContent().build();
    }

    @PostMapping("/{id}/retry")
    public ResponseEntity<Void> retry(@PathVariable UUID id) {
        retryProcessing.retry(id);
        return ResponseEntity.accepted().build();
    }

    @GetMapping("/{id}/metadata")
    public ResponseEntity<MetadataResponse> metadata(@PathVariable UUID id) {
        return processingResultRepository.findMetadata(id)
                .map(m -> ResponseEntity.ok(new MetadataResponse(
                        m.documentId(), m.pageCount(),
                        m.pdfCreatedAt(), m.pdfModifiedAt(), m.pdfAuthor(), m.pdfTitle(),
                        m.imageCapturedAt(), m.imageDevice(),
                        m.imageGpsPresent(), m.imageGpsRedacted(),
                        m.detectedLanguage())))
                .orElse(ResponseEntity.notFound().build());
    }

    @GetMapping("/{id}/text")
    public ResponseEntity<ExtractedTextResponse> text(@PathVariable UUID id) {
        List<ExtractedPage> pages = processingResultRepository.findExtractedPages(id);
        if (pages.isEmpty()) {
            return ResponseEntity.notFound().build();
        }
        String combinedText = pages.stream()
                .map(ExtractedPage::text)
                .collect(java.util.stream.Collectors.joining("\n\n"));
        List<ExtractedPageResponse> pageResponses = pages.stream()
                .map(p -> new ExtractedPageResponse(p.pageNumber(), p.text()))
                .toList();
        return ResponseEntity.ok(new ExtractedTextResponse(id, pageResponses, combinedText));
    }

    @GetMapping("/{id}/thumbnail")
    public ResponseEntity<InputStreamResource> thumbnail(
            @PathVariable UUID id,
            @RequestParam(defaultValue = "SMALL") String size
    ) {
        return processingResultRepository.findThumbnail(id, size.toUpperCase())
                .map(thumb -> {
                    InputStream stream = documentStorage.retrieve(thumb.storageKey());
                    return ResponseEntity.ok()
                            .contentType(MediaType.IMAGE_JPEG)
                            .header(HttpHeaders.CACHE_CONTROL, "public, max-age=31536000, immutable")
                            .body(new InputStreamResource(stream));
                })
                .orElse(ResponseEntity.notFound().build());
    }

    public record UploadResult(UUID id, String fileName, String status, String error) {
        static UploadResult success(UUID id, String status) {
            return new UploadResult(id, null, status, null);
        }

        static UploadResult failure(String fileName, String error) {
            return new UploadResult(null, fileName, "FAILED", error);
        }
    }

    public record DocumentSummary(
            UUID id, String fileName, String contentType,
            long sizeBytes, String status, OffsetDateTime createdAt) {}

    public record ListDocumentsResponse(
            List<DocumentSummary> items, int page, int size,
            long totalElements, boolean hasNext) {}

    public record DocumentDetailResponse(
            UUID id, String fileName, String contentType, long sizeBytes,
            String sha256, String status, String failureReason,
            OffsetDateTime createdAt, OffsetDateTime updatedAt) {}

    public record UploadConstraintsResponse(
            Set<String> supportedContentTypes,
            long maxFileSize,
            String maxFileSizeLabel,
            int maxFilesPerRequest) {}

    public record MetadataResponse(
            UUID documentId, Integer pageCount,
            OffsetDateTime pdfCreatedAt, OffsetDateTime pdfModifiedAt,
            String pdfAuthor, String pdfTitle,
            OffsetDateTime imageCapturedAt, String imageDevice,
            boolean imageGpsPresent, boolean imageGpsRedacted,
            String detectedLanguage) {}

    public record ExtractedTextResponse(
            UUID documentId,
            List<ExtractedPageResponse> pages,
            String combinedText) {}

    public record ExtractedPageResponse(int pageNumber, String text) {}
}
