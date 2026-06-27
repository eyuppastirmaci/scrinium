package dev.scrinium.document.application;

import dev.scrinium.document.domain.port.in.MarkDocumentReady;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import dev.scrinium.document.domain.port.out.ProcessingResultRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

@Service
public class MarkDocumentReadyService implements MarkDocumentReady {

    private static final Logger log = LoggerFactory.getLogger(MarkDocumentReadyService.class);

    private final DocumentRepository documentRepository;
    private final ProcessingResultRepository processingResultRepository;

    public MarkDocumentReadyService(
            DocumentRepository documentRepository,
            ProcessingResultRepository processingResultRepository
    ) {
        this.documentRepository = documentRepository;
        this.processingResultRepository = processingResultRepository;
    }

    @Override
    @Transactional
    public void markReady(ProcessingResult result) {
        var documentId = result.documentId();

        if (!result.pages().isEmpty()) {
            processingResultRepository.saveExtractedPages(documentId, result.pages());
            log.info("Saved {} extracted pages for document {}", result.pages().size(), documentId);
        }

        if (result.metadata() != null) {
            processingResultRepository.saveMetadata(result.metadata());
            log.info("Saved metadata for document {}", documentId);
        }

        if (!result.thumbnails().isEmpty()) {
            processingResultRepository.saveThumbnails(documentId, result.thumbnails());
            log.info("Saved {} thumbnails for document {}", result.thumbnails().size(), documentId);
        }

        int updated = documentRepository.markReadyIfPending(documentId);

        if (updated == 1) {
            log.info("Document {} marked READY", documentId);
        } else {
            log.info("Document {} not transitioned (already READY or unknown)", documentId);
        }
    }
}