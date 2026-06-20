package dev.scrinium.document.application;

import dev.scrinium.document.domain.exception.DocumentNotFoundException;
import dev.scrinium.document.domain.port.in.DeleteDocument;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

@Service
public class DeleteDocumentService implements DeleteDocument {

    private final DocumentRepository repository;
    private final DocumentEventPublisher eventPublisher;

    public DeleteDocumentService(DocumentRepository repository,
                                 DocumentEventPublisher eventPublisher) {
        this.repository = repository;
        this.eventPublisher = eventPublisher;
    }

    @Override
    @Transactional
    public void delete(UUID documentId) {
        int affected = repository.markDeleted(documentId);
        if (affected == 0) {
            throw new DocumentNotFoundException(documentId);
        }

        eventPublisher.documentDeleted(documentId);
    }
}
