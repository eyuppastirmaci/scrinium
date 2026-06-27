package dev.scrinium.document.application;

import dev.scrinium.document.domain.exception.DocumentNotFoundException;
import dev.scrinium.document.domain.model.DocumentThumbnail;
import dev.scrinium.document.domain.port.in.DeleteDocument;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import dev.scrinium.document.domain.port.out.DocumentStorage;
import dev.scrinium.document.domain.port.out.ProcessingResultRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

@Service
public class DeleteDocumentService implements DeleteDocument {

    private static final Logger log = LoggerFactory.getLogger(DeleteDocumentService.class);

    private final DocumentRepository repository;
    private final ProcessingResultRepository processingResultRepository;
    private final DocumentStorage documentStorage;
    private final DocumentEventPublisher eventPublisher;

    public DeleteDocumentService(DocumentRepository repository,
                                 ProcessingResultRepository processingResultRepository,
                                 DocumentStorage documentStorage,
                                 DocumentEventPublisher eventPublisher) {
        this.repository = repository;
        this.processingResultRepository = processingResultRepository;
        this.documentStorage = documentStorage;
        this.eventPublisher = eventPublisher;
    }

    @Override
    @Transactional
    public void delete(UUID documentId) {
        List<DocumentThumbnail> thumbnails = processingResultRepository.findThumbnails(documentId);

        processingResultRepository.deleteAll(documentId);

        int affected = repository.markDeleted(documentId);
        if (affected == 0) {
            throw new DocumentNotFoundException(documentId);
        }

        // Best-effort: delete thumbnail files from object storage outside the transaction.
        for (DocumentThumbnail thumb : thumbnails) {
            try {
                documentStorage.delete(thumb.storageKey());
            } catch (Exception e) {
                log.warn("Failed to delete thumbnail {} from storage: {}", thumb.storageKey(), e.getMessage());
            }
        }

        eventPublisher.documentDeleted(documentId);

        log.info("Document {} deleted with {} thumbnails cleaned up", documentId, thumbnails.size());
    }
}
