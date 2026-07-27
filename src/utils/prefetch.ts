import type { DocFull } from "../types";

/** 预加载文档内容的内存缓存 */
const cache = new Map<string, DocFull | Promise<DocFull | null>>();

export function prefetchDoc(
    id: string,
): Promise<DocFull | null> | DocFull | undefined {
    if (cache.has(id)) return cache.get(id);

    const promise = fetch(`/api/cats/${id}`)
        .then((res) => res.json())
        .then((data) => {
            if (data.success) {
                const doc: DocFull = {
                    ...data.data,
                    locked: data.data.locked ?? false,
                };
                cache.set(id, doc);
                return doc;
            }
            return null;
        })
        .catch(() => null);

    cache.set(id, promise);
    return promise;
}

export function getCachedDoc(id: string): DocFull | undefined {
    const entry = cache.get(id);
    if (entry && !(entry instanceof Promise)) {
        return entry as DocFull;
    }
    return undefined;
}

export function clearCache(): void {
    cache.clear();
}
