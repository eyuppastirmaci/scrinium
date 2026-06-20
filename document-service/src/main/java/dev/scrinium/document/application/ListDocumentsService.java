package dev.scrinium.document.application;

import dev.scrinium.document.domain.model.Document;
import dev.scrinium.document.domain.model.DocumentPage;
import dev.scrinium.document.domain.port.in.ListDocuments;
import dev.scrinium.document.domain.port.out.DocumentRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

@Service
public class ListDocumentsService implements ListDocuments {

    private final DocumentRepository repository;

    public ListDocumentsService(DocumentRepository repository) {
        this.repository = repository;
    }

    /**
     * Lists non-deleted documents with pagination.
     *
     * <p>{@code readOnly = true} serves two purposes: it wraps both queries ({@code findAll} and
     * {@code countAccessible}) in a single read-only transaction so the count stays consistent with
     * the fetched page, and it hints PostgreSQL to skip write-ahead log overhead since no mutations
     * will occur.</p>
     */
    @Override
    @Transactional(readOnly = true)
    public DocumentPage list(Query query) {
        // Convert zero-based page number to a row offset for the SQL LIMIT/OFFSET query.
        int offset = query.page() * query.size();

        // Fetch one page of non-deleted documents, ordered by creation date descending.
        List<Document> items = repository.findAll(offset, query.size());

        // Count all accessible documents to calculate total pages on the frontend.
        long totalElements = repository.countAccessible();

        // Determine whether more pages exist beyond the current one.
        boolean hasNext = (long) offset + query.size() < totalElements;

        return new DocumentPage(items, query.page(), query.size(), totalElements, hasNext);
    }
}
