package dev.scrinium.document.application;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.port.in.UploadDocument;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.time.OffsetDateTime;
import java.util.UUID;

@Service
public class UploadDocumentService implements UploadDocument {

    private final DocumentRepository repository;
    private final DocumentEventPublisher eventPublisher;

    public UploadDocumentService(DocumentRepository repository,
                                 DocumentEventPublisher eventPublisher
    ) {
        this.repository = repository;
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
        // Create an always-valid PENDING aggregate; invariants (e.g. non-blank fileName) are enforced in its constructor.
        Document document = Document.pending(
                UUID.randomUUID(),
                command.fileName(),
                OffsetDateTime.now()
        );

        // Persist through the outbound port; the concrete PostgreSQL adapter stays hidden behind the interface.
        repository.save(document);

        // Announce the upload through the outbound port; the concrete Kafka adapter stays hidden behind the interface.
        eventPublisher.documentUploaded(document);

        return document;
    }
}