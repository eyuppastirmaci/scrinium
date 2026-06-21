package dev.scrinium.document.application;

import dev.scrinium.document.domain.port.in.MarkDocumentFailed;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

@Service
public class MarkDocumentFailedService implements MarkDocumentFailed {

    private static final Logger log = LoggerFactory.getLogger(MarkDocumentFailedService.class);

    private final DocumentRepository repository;

    public MarkDocumentFailedService(DocumentRepository repository) {
        this.repository = repository;
    }

    @Override
    @Transactional
    public void markFailed(UUID documentId) {
        int updated = repository.markFailedIfPending(documentId);

        if (updated == 1) {
            log.info("Document {} marked FAILED", documentId);
        } else {
            log.info("Document {} not transitioned to FAILED (already READY/FAILED or unknown)", documentId);
        }
    }
}
