package dev.scrinium.document.application;

import dev.scrinium.document.domain.exception.DocumentNotFoundException;
import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentDownload;
import dev.scrinium.document.domain.port.in.DownloadDocument;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import dev.scrinium.document.domain.port.out.DocumentStorage;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.io.InputStream;
import java.util.UUID;

@Service
public class DownloadDocumentService implements DownloadDocument {

    private final DocumentRepository repository;
    private final DocumentStorage storage;

    public DownloadDocumentService(DocumentRepository repository, DocumentStorage storage) {
        this.repository = repository;
        this.storage = storage;
    }

    @Override
    @Transactional(readOnly = true)
    public DocumentDownload download(UUID documentId) {
        Document document = repository.findById(documentId)
                .orElseThrow(() -> new DocumentNotFoundException(documentId));

        InputStream content = storage.retrieve(document.storageObjectKey());

        return new DocumentDownload(
                document.fileName(),
                document.contentType(),
                document.sizeBytes(),
                content
        );
    }
}
