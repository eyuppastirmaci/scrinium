package dev.scrinium.document.adapter.in.messaging;

import dev.scrinium.document.domain.port.in.MarkDocumentFailed;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.kafka.annotation.KafkaListener;
import org.springframework.stereotype.Component;
import tools.jackson.databind.json.JsonMapper;
import tools.jackson.databind.JsonNode;

import java.util.UUID;

@Component
public class DocumentProcessingFailedListener {

    private static final Logger log = LoggerFactory.getLogger(DocumentProcessingFailedListener.class);

    private final MarkDocumentFailed markDocumentFailed;
    private final JsonMapper jsonMapper;

    public DocumentProcessingFailedListener(
            MarkDocumentFailed markDocumentFailed,
            JsonMapper jsonMapper
    ) {
        this.markDocumentFailed = markDocumentFailed;
        this.jsonMapper = jsonMapper;
    }

    @KafkaListener(topics = "document.processing.failed", groupId = "document-service")
    public void onMessage(String message) {
        JsonNode root = jsonMapper.readTree(message);
        UUID documentId = UUID.fromString(root.get("payload").get("documentId").asText());
        String reason = root.get("payload").get("reason").asText();

        log.info("Received document.processing.failed for {}: {}", documentId, reason);

        markDocumentFailed.markFailed(documentId);
    }
}
