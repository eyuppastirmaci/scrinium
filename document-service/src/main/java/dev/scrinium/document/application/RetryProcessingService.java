package dev.scrinium.document.application;

import dev.scrinium.document.domain.exception.DocumentNotFoundException;
import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.port.in.RetryProcessing;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

@Service
public class RetryProcessingService implements RetryProcessing {

    private static final Logger log = LoggerFactory.getLogger(RetryProcessingService.class);

    private final DocumentRepository repository;
    private final DocumentEventPublisher eventPublisher;

    public RetryProcessingService(DocumentRepository repository, DocumentEventPublisher eventPublisher) {
        this.repository = repository;
        this.eventPublisher = eventPublisher;
    }

    @Override
    @Transactional
    public void retry(UUID documentId) {
        int updated = repository.markPendingIfFailed(documentId);

        if (updated == 0) {
            throw new DocumentNotFoundException(documentId);
        }

        Document document = repository.findById(documentId)
                .orElseThrow(() -> new DocumentNotFoundException(documentId));

        eventPublisher.documentUploaded(document);

        log.info("Document {} queued for reprocessing", documentId);
    }
}
