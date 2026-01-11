export function formatDate(isoStr: string) {
    const timestamp = Date.parse(isoStr);
    return new Date(timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})
}