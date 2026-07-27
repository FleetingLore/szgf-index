import React, { useEffect, useState, useCallback } from "react";
import Listing from "./pages/Listing";
import DocPage from "./pages/DocPage";
import AdminPage from "./pages/AdminPage";
import { useAdmin } from "./hooks/useAdmin";
import { useCats } from "./hooks/useCats";
import { useUpload } from "./hooks/useUpload";
import { downloadDoc } from "./utils/download";
import { getCachedDoc, prefetchDoc, clearCache } from "./utils/prefetch";
import { viewRegistry } from "./views/registry";
import { registerDefaultViews } from "./views/defaults";
import { registerDefaultFormats } from "./rendering/defaults";
import type { DocFull, DocListItem } from "./types";
import type { GetCatResponse, ListCatsResponse } from "./types/api";

// 应用启动时注册默认视图和格式
registerDefaultViews();
registerDefaultFormats();

function App() {
    const [viewName, setViewName] = useState<string>("loading");
    const [docId, setDocId] = useState<string | null>(null);
    const {
        isAdmin,
        loading: adminLoading,
        createSession,
        destroySession,
    } = useAdmin();
    const { cats, siteName, loading: catsLoading, loadCats } = useCats();
    const [currentDoc, setCurrentDoc] = useState<DocFull | null>(null);
    const { upload } = useUpload(loadCats);

    useEffect(() => {
        const path = location.pathname;
        // 通过注册表解析视图
        const view = viewRegistry.resolve(path);
        if (view) {
            setViewName(view.name);
            // 提取文档 ID
            const m = path.match(/^\/listing\/lib\/(\d+)$/);
            if (m) setDocId(m[1]);
        }
    }, []);

    const loadDocWithCache = useCallback(async (id: string) => {
        let doc = getCachedDoc(id);
        if (doc) {
            setCurrentDoc(doc);
            return;
        }
        setCurrentDoc({
            id,
            title: "...",
            file: `${id}.md`,
            content: "",
            created_at: Date.now(),
            deprecated: false,
            locked: false,
        });
        try {
            const res = await fetch(`/api/cats/${id}`);
            const data: GetCatResponse = await res.json();
            if (data.success && data.data) {
                setCurrentDoc({
                    ...data.data,
                    locked: data.data.locked ?? false,
                });
            } else {
                goHome();
            }
        } catch {
            goHome();
        }
    }, []);

    useEffect(() => {
        if (viewName === "doc" && docId) {
            loadDocWithCache(docId);
        }
    }, [viewName, docId, loadDocWithCache]);

    const loadCatsAndPrefetch = useCallback(async () => {
        const res = await fetch("/api/cats");
        const data: ListCatsResponse = await res.json();
        if (data.success && data.data) {
            data.data
                .slice(0, 5)
                .forEach((doc: DocListItem) => prefetchDoc(doc.id));
        }
    }, []);

    useEffect(() => {
        if (viewName === "listing") loadCatsAndPrefetch();
    }, [viewName, loadCatsAndPrefetch]);

    const goHome = () => {
        location.href = "/";
    };
    const goAdmin = () => {
        if (isAdmin) {
            destroySession();
        } else {
            location.href = "/listing/lib/0";
        }
    };
    const handleAdminSuccess = (token: string) => {
        createSession(token);
        goHome();
    };

    if (adminLoading || (viewName === "listing" && catsLoading)) {
        return <div></div>;
    }

    if (viewName === "admin") {
        return <AdminPage onSuccess={handleAdminSuccess} />;
    }

    if (viewName === "listing") {
        return (
            <Listing
                siteName={siteName}
                cats={cats}
                isAdmin={isAdmin}
                onUpload={upload}
                onAdminClick={goAdmin}
            />
        );
    }

    if (viewName === "doc" && currentDoc) {
        return (
            <DocPage
                doc={currentDoc}
                isAdmin={isAdmin}
                onDownload={() => downloadDoc(currentDoc)}
                onDeprecate={() => {
                    const newVal = !currentDoc.deprecated;
                    setCurrentDoc({ ...currentDoc, deprecated: newVal });
                    fetch(`/api/cats/${currentDoc.id}/deprecated`, {
                        method: "PUT",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ deprecated: newVal }),
                    }).then(() => {
                        clearCache();
                        loadDocWithCache(currentDoc.id);
                        loadCats();
                    });
                }}
                onLock={() => {
                    const newVal = !currentDoc.locked;
                    setCurrentDoc({ ...currentDoc, locked: newVal });
                    fetch(`/api/cats/${currentDoc.id}/lock`, {
                        method: "PUT",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ locked: newVal }),
                    }).then(() => {
                        clearCache(); // 清除旧缓存，下次加载拉最新数据
                        loadDocWithCache(currentDoc.id);
                    });
                }}
                onDelete={() => {
                    fetch(`/api/cats/${currentDoc.id}/deleted`, {
                        method: "PUT",
                    }).then(() => goHome());
                }}
                onHome={goHome}
            />
        );
    }

    return <div></div>;
}

export default App;
