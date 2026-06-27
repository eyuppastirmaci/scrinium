package dev.scrinium.document.domain.model;

import java.util.UUID;

public record DocumentThumbnail(
        UUID documentId,
        String size,
        String storageKey,
        int width,
        int height
) {}
