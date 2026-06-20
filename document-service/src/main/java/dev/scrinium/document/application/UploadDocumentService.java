package dev.scrinium.document.application;

import dev.scrinium.document.adapter.in.web.config.UploadProperties;
import dev.scrinium.document.domain.exception.DuplicateDocumentException;
import dev.scrinium.document.domain.exception.UnsupportedFileTypeException;
import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentFile;
import dev.scrinium.document.domain.model.StoredDocument;
import dev.scrinium.document.domain.port.in.UploadDocument;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import dev.scrinium.document.domain.port.out.DocumentStorage;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.OffsetDateTime;
import java.util.UUID;

@Service
public class UploadDocumentService implements UploadDocument {

    private final UploadProperties uploadProperties;
    private final DocumentRepository repository;
    private final DocumentStorage storage;
    private final DocumentEventPublisher eventPublisher;

    public UploadDocumentService(UploadProperties uploadProperties,
                                 DocumentRepository repository,
                                 DocumentStorage storage,
                                 DocumentEventPublisher eventPublisher
    ) {
        this.uploadProperties = uploadProperties;
        this.repository = repository;
        this.storage = storage;
        this.eventPublisher = eventPublisher;
    }

    /**
     * Handles the <em>upload document</em> use case: creates a new {@link Document} in the
     * {@code PENDING} state, persists it, and announces it with a {@code document.uploaded} event.
     *
     * <p>Both side effects run inside a single transaction via {@link Transactional}. Note that the
     * database write and the Kafka publish are a <strong>dual write</strong> and are <em>not</em>
     * atomic; for at-least-once guarantees this should later be replaced by the Outbox pattern.</p>
     *
     * <p>This method only orchestrates: the business rules (e.g. a non-blank file name) live inside
     * the {@link Document} aggregate, and persistence/messaging are reached through outbound ports
     * rather than concrete adapters.</p>
     *
     * @param command the inbound command carrying the data required to create the document
     * @return the newly created {@link Document}, so the web adapter can build its HTTP response
     */
    @Override
    @Transactional
    public Document upload(Command command) {
        // Reject unsupported file types before any storage or persistence work.
        if (!uploadProperties.supportedContentTypes().contains(command.contentType())) {
            throw new UnsupportedFileTypeException(command.contentType());
        }

        // Generate a unique identifier for the new document aggregate.
        UUID documentId = UUID.randomUUID();

        // Stream the file to MinIO and compute its SHA-256 hash on the fly.
        StoredDocument storedDocument = storage.store(new DocumentFile(
                documentId,
                command.fileName(),
                command.contentType(),
                command.sizeBytes(),
                command.content()
        ));

        // Check whether a non-deleted document with the same content already exists.
        repository.findBySha256(storedDocument.sha256())
                .ifPresent(existing -> {
                    throw new DuplicateDocumentException(existing.id());
                });

        // Create an always-valid PENDING aggregate; invariants (e.g. non-blank fileName) are enforced in its constructor.
        Document document = Document.pending(
                documentId,
                command.fileName(),
                storedDocument.contentType(),
                storedDocument.sizeBytes(),
                storedDocument.objectKey(),
                storedDocument.sha256(),
                OffsetDateTime.now()
        );

        // Persist through the outbound port; the concrete PostgreSQL adapter stays hidden behind the interface.
        repository.save(document);

        // Announce the upload through the outbound port; the concrete Kafka adapter stays hidden behind the interface.
        eventPublisher.documentUploaded(document);

        return document;
    }
}
