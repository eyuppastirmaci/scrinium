package dev.scrinium.document.domain.port.in;

import dev.scrinium.document.domain.model.DocumentPage;

public interface ListDocuments {
    DocumentPage list(Query query);

    record Query(int page, int size) {
        public Query {
            if (page < 0) page = 0;
            if (size <= 0 || size > 100) size = 20;
        }
    }
}
