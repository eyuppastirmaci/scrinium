package dev.scrinium.document.domain.model;

import java.time.OffsetDateTime;
import java.util.UUID;

public record DocumentMetadata(
        UUID documentId,
        Integer pageCount,
        OffsetDateTime pdfCreatedAt,
        OffsetDateTime pdfModifiedAt,
        String pdfAuthor,
        String pdfTitle,
        OffsetDateTime imageCapturedAt,
        String imageDevice,
        boolean imageGpsPresent,
        boolean imageGpsRedacted,
        String detectedLanguage
) {}
