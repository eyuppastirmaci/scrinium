package dev.scrinium.document.common;

public final class FormatUtils {

    private FormatUtils() {}

    /**
     * Formats a byte count as a human-readable megabyte string (e.g. "20 MB").
     * Returns a fallback message when the value is zero or negative.
     */
    public static String toMegabytes(long bytes) {
        return bytes > 0 ? (bytes / (1024 * 1024)) + " MB" : "the configured limit";
    }
}
