package dev.scrinium.document.adapter.in.web;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentDownload;
import dev.scrinium.document.domain.model.DocumentPage;
import dev.scrinium.document.domain.port.in.DeleteDocument;
import dev.scrinium.document.domain.port.in.DownloadDocument;
import dev.scrinium.document.domain.port.in.GetDocument;
import dev.scrinium.document.domain.port.in.ListDocuments;
import dev.scrinium.document.domain.port.in.UploadDocument;
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

import java.time.OffsetDateTime;
import java.util.List;
import java.util.UUID;

@RestController
@RequestMapping("/documents")
public class DocumentController {

    private final UploadDocument uploadDocument;
    private final ListDocuments listDocuments;
    private final GetDocument getDocument;
    private final DownloadDocument downloadDocument;
    private final DeleteDocument deleteDocument;
    private final DocumentUploadRequestMapper uploadRequestMapper;

    public DocumentController(UploadDocument uploadDocument,
                              ListDocuments listDocuments,
                              GetDocument getDocument,
                              DownloadDocument downloadDocument,
                              DeleteDocument deleteDocument,
                              DocumentUploadRequestMapper uploadRequestMapper) {
        this.uploadDocument = uploadDocument;
        this.listDocuments = listDocuments;
        this.getDocument = getDocument;
        this.downloadDocument = downloadDocument;
        this.deleteDocument = deleteDocument;
        this.uploadRequestMapper = uploadRequestMapper;
    }

    @PostMapping(consumes = "multipart/form-data")
    public ResponseEntity<UploadResponse> upload(
            @RequestParam("file") MultipartFile file
    ) {
        Document document = uploadDocument.upload(uploadRequestMapper.toCommand(file));

        return ResponseEntity.accepted()
                .body(new UploadResponse(document.id(), document.status().name()));
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
                d.sha256(), d.status().name(), d.createdAt(), d.updatedAt());
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

    public record UploadResponse(UUID id, String status) {}

    public record DocumentSummary(
            UUID id, String fileName, String contentType,
            long sizeBytes, String status, OffsetDateTime createdAt) {}

    public record ListDocumentsResponse(
            List<DocumentSummary> items, int page, int size,
            long totalElements, boolean hasNext) {}

    public record DocumentDetailResponse(
            UUID id, String fileName, String contentType, long sizeBytes,
            String sha256, String status,
            OffsetDateTime createdAt, OffsetDateTime updatedAt) {}
}
