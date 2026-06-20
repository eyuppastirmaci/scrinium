package dev.scrinium.document.application;

import dev.scrinium.document.domain.exception.DocumentNotFoundException;
import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.port.in.GetDocument;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.UUID;

@Service
public class GetDocumentService implements GetDocument {

    private final DocumentRepository repository;

    public GetDocumentService(DocumentRepository repository) {
        this.repository = repository;
    }

    @Override
    @Transactional(readOnly = true)
    public Document get(UUID documentId) {
        return repository.findById(documentId)
                .orElseThrow(() -> new DocumentNotFoundException(documentId));
    }
}
