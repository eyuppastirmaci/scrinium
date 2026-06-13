package dev.scrinium.document.adapter.in.messaging;

import dev.scrinium.document.domain.port.in.MarkDocumentReady;
import dev.scrinium.document.events.generated.DocumentProcessingCompleted;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.kafka.annotation.KafkaListener;
import org.springframework.stereotype.Component;
import tools.jackson.databind.json.JsonMapper;

import java.util.UUID;

@Component
public class DocumentProcessingCompletedListener {

    private static final Logger log = LoggerFactory.getLogger(DocumentProcessingCompletedListener.class);

    private final MarkDocumentReady markDocumentReady;
    private final JsonMapper jsonMapper;

    public DocumentProcessingCompletedListener(
            MarkDocumentReady markDocumentReady,
            JsonMapper jsonMapper
    ) {
        this.markDocumentReady = markDocumentReady;
        this.jsonMapper = jsonMapper;
    }

    @KafkaListener(topics = "document.processing.completed", groupId = "document-service")
    public void onMessage(String message) {
        // Deserialize the contract event ourselves (symmetric with the publisher side).
        DocumentProcessingCompleted event = jsonMapper.readValue(message, DocumentProcessingCompleted.class);

        UUID documentId = event.getPayload().getDocumentId();

        log.info("Received document.processing.completed for {}", documentId);

        // If this throws, the container won't commit the offset -> at-least-once redelivery.
        markDocumentReady.markReady(documentId);
    }
}