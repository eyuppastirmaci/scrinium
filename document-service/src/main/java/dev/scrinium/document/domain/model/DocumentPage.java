package dev.scrinium.document.domain.model;

import java.util.List;

public record DocumentPage(
        List<Document> items,
        int page,
        int size,
        long totalElements,
        boolean hasNext
) {
    public DocumentPage {
        if (page < 0) throw new IllegalArgumentException("page must not be negative");
        if (size <= 0) throw new IllegalArgumentException("size must be positive");
        if (totalElements < 0) throw new IllegalArgumentException("totalElements must not be negative");
        items = List.copyOf(items);
    }
}
