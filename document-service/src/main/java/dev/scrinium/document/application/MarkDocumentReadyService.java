package dev.scrinium.document.application;

import dev.scrinium.document.domain.port.in.MarkDocumentReady;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

@Service
public class MarkDocumentReadyService implements MarkDocumentReady {

    private static final Logger log = LoggerFactory.getLogger(MarkDocumentReadyService.class);

    private final DocumentRepository repository;

    public MarkDocumentReadyService(DocumentRepository repository) {
        this.repository = repository;
    }

    @Override
    @Transactional
    public void markReady(UUID documentId) {
        int updated = repository.markReadyIfPending(documentId);

        if (updated == 1) {
            log.info("Document {} marked READY", documentId);
        } else {
            // Idempotent no-op: already READY or unknown id (e.g. a duplicate delivery).
            log.info("Document {} not transitioned (already READY or unknown)", documentId);
        }
    }
}