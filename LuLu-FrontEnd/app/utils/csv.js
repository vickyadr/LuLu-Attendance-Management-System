// A quoted CSV cell is still evaluated by Excel, so text that starts with a
// formula trigger character gets a leading apostrophe. Numbers stay numbers.
export function csvCell(v) {
    let text = String(v ?? "");
    if (typeof v !== "number" && /^[=+\-@\t\r]/.test(text)) text = "'" + text;
    return `"${text.replace(/"/g, '""')}"`;
}
