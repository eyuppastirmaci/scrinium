package dev.scrinium.document.adapter.out.messaging;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.port.out.DocumentEventPublisher;
import dev.scrinium.document.events.generated.DocumentUploaded;
import dev.scrinium.document.events.generated.DocumentUploadedPayload;
import org.springframework.kafka.core.KafkaTemplate;
import org.springframework.stereotype.Component;
import tools.jackson.databind.json.JsonMapper;

import java.time.OffsetDateTime;
import java.util.UUID;

@Component
public class KafkaDocumentEventPublisher implements DocumentEventPublisher {

    private static final String TOPIC = "document.uploaded";

    private static final String EVENT_TYPE = "document.uploaded";
    private static final long EVENT_VERSION = 1L;

    private final KafkaTemplate<String, String> kafkaTemplate;
    private final JsonMapper jsonMapper;

    public KafkaDocumentEventPublisher(KafkaTemplate<String, String> kafkaTemplate,
                                       JsonMapper jsonMapper) {
        this.kafkaTemplate = kafkaTemplate;
        this.jsonMapper = jsonMapper;
    }

    /**
     * Publishes a <strong>{@code document.uploaded}</strong> event to Kafka after a document
     * has been persisted in the {@code PENDING} state.
     *
     * <p>The event is built from the {@link DocumentUploaded} type that is code-generated from the
     * shared JSON Schema contract, then serialized to plain contract JSON. Serialization is done
     * here on purpose &mdash; using a {@code String} value and our own {@code JsonMapper} keeps the
     * wire payload free of Spring-specific type headers, which matters for the polyglot (Rust)
     * consumer.</p>
     *
     * <p>The message is keyed by {@code documentId} so that all events for a single document are
     * routed to the same partition and therefore keep their relative order.</p>
     *
     * <p><em>Note:</em> this is the outbound-port implementation; the rest of the application depends
     * only on {@link dev.scrinium.document.domain.port.out.DocumentEventPublisher}, never on Kafka.</p>
     *
     * @param document the persisted document the event refers to; its metadata populates the payload
     */
    @Override
    public void documentUploaded(Document document) {
        // Build the contract event (type generated from the shared JSON Schema).
        DocumentUploaded event = new DocumentUploaded()
                // Unique per-message id, separate from the document id; used for consumer-side idempotency.
                .withId(UUID.randomUUID())
                // Type discriminator so polyglot consumers can route/filter the message.
                .withType(EVENT_TYPE)
                // Schema major version, letting consumers cope with future payload evolution.
                .withVersion(EVENT_VERSION)
                // Event creation time (offset-aware), serialized as ISO-8601 by Jackson 3.
                .withTimestamp(OffsetDateTime.now())
                // Event-specific body.
                .withPayload(new DocumentUploadedPayload()
                        // The aggregate this event refers to (links back to the persisted row).
                        .withDocumentId(document.id())
                        // Original and storage metadata the processing-service needs to read the file.
                        .withFileName(document.fileName())
                        .withContentType(document.contentType())
                        .withSizeBytes(document.sizeBytes())
                        .withStorageObjectKey(document.storageObjectKey())
                        .withSha256(document.sha256()));

        // Serialize to plain contract JSON ourselves (no Spring type headers); Jackson 3 throws unchecked.
        String json = jsonMapper.writeValueAsString(event);

        // Publish keyed by documentId so all events for one document keep their order across partitions.
        kafkaTemplate.send(TOPIC, document.id().toString(), json);
    }
}
