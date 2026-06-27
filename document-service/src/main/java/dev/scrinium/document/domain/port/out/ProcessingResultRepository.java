package dev.scrinium.document.domain.port.out;

import dev.scrinium.document.domain.model.DocumentMetadata;
import dev.scrinium.document.domain.model.DocumentThumbnail;
import dev.scrinium.document.domain.model.ExtractedPage;

import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface ProcessingResultRepository {

    void saveExtractedPages(UUID documentId, List<ExtractedPage> pages);

    List<ExtractedPage> findExtractedPages(UUID documentId);

    void saveMetadata(DocumentMetadata metadata);

    Optional<DocumentMetadata> findMetadata(UUID documentId);

    void saveThumbnails(UUID documentId, List<DocumentThumbnail> thumbnails);

    Optional<DocumentThumbnail> findThumbnail(UUID documentId, String size);

    List<DocumentThumbnail> findThumbnails(UUID documentId);

    void deleteAll(UUID documentId);
}
