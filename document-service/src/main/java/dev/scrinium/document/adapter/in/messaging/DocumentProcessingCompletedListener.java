package dev.scrinium.document.adapter.in.messaging;

import dev.scrinium.document.domain.model.DocumentMetadata;
import dev.scrinium.document.domain.model.DocumentThumbnail;
import dev.scrinium.document.domain.model.ExtractedPage;
import dev.scrinium.document.domain.port.in.MarkDocumentReady;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.kafka.annotation.KafkaListener;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
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
        JsonNode root = jsonMapper.readTree(message);
        JsonNode payload = root.get("payload");
        UUID documentId = UUID.fromString(payload.get("documentId").asText());

        log.info("Received document.processing.completed for {}", documentId);

        List<ExtractedPage> pages = parsePages(payload);
        DocumentMetadata metadata = parseMetadata(documentId, payload);
        List<DocumentThumbnail> thumbnails = parseThumbnails(documentId, payload);

        markDocumentReady.markReady(new MarkDocumentReady.ProcessingResult(
                documentId, pages, metadata, thumbnails
        ));
    }

    private List<ExtractedPage> parsePages(JsonNode payload) {
        JsonNode pagesNode = payload.get("pages");
        if (pagesNode == null || !pagesNode.isArray()) return List.of();

        List<ExtractedPage> pages = new ArrayList<>();
        for (JsonNode page : pagesNode) {
            pages.add(new ExtractedPage(
                    page.get("pageNumber").asInt(),
                    page.get("text").asText()
            ));
        }
        return pages;
    }

    private DocumentMetadata parseMetadata(UUID documentId, JsonNode payload) {
        JsonNode m = payload.get("metadata");
        if (m == null) return null;

        return new DocumentMetadata(
                documentId,
                intOrNull(m, "pageCount"),
                dateOrNull(m, "pdfCreatedAt"),
                dateOrNull(m, "pdfModifiedAt"),
                textOrNull(m, "pdfAuthor"),
                textOrNull(m, "pdfTitle"),
                dateOrNull(m, "imageCapturedAt"),
                textOrNull(m, "imageDevice"),
                m.path("imageGpsPresent").asBoolean(false),
                m.path("imageGpsRedacted").asBoolean(false),
                textOrNull(m, "detectedLanguage")
        );
    }

    private List<DocumentThumbnail> parseThumbnails(UUID documentId, JsonNode payload) {
        JsonNode thumbsNode = payload.get("thumbnails");
        if (thumbsNode == null || !thumbsNode.isArray()) return List.of();

        List<DocumentThumbnail> thumbnails = new ArrayList<>();
        for (JsonNode t : thumbsNode) {
            thumbnails.add(new DocumentThumbnail(
                    documentId,
                    t.get("size").asText(),
                    t.get("storageKey").asText(),
                    t.get("width").asInt(),
                    t.get("height").asInt()
            ));
        }
        return thumbnails;
    }

    private static String textOrNull(JsonNode node, String field) {
        JsonNode v = node.get(field);
        return (v != null && !v.isNull()) ? v.asText() : null;
    }

    private static Integer intOrNull(JsonNode node, String field) {
        JsonNode v = node.get(field);
        return (v != null && !v.isNull()) ? v.asInt() : null;
    }

    private static OffsetDateTime dateOrNull(JsonNode node, String field) {
        JsonNode v = node.get(field);
        return (v != null && !v.isNull()) ? OffsetDateTime.parse(v.asText()) : null;
    }
}