import React, { useState, useEffect } from "react";
import { useI18n } from "../hooks/useI18n";
import { formatRegistry } from "../rendering/registry";
import DocHeader from "../components/DocHeader";
import DocFooter from "../components/DocFooter";
import DeprecatedBanner from "../components/DeprecatedBanner";
import type { DocFull } from "../types";
import styles from "./DocPage.module.css";

interface DocPageProps {
    doc: DocFull;
    isAdmin: boolean;
    onDownload: () => void;
    onDeprecate: () => void;
    onLock: () => void;
    onDelete: () => void;
    onHome: () => void;
}

export default function DocPage({
    doc,
    isAdmin,
    onDownload,
    onDeprecate,
    onLock,
    onDelete,
    onHome,
}: DocPageProps) {
    const { t } = useI18n();
    const [content, setContent] = useState(doc.content || "");

    useEffect(() => {
        if (doc.content) setContent(doc.content);
    }, [doc.content]);

    // 通过格式注册表查找渲染器（按文件名匹配，fallback 到默认）
    const renderer =
        formatRegistry.resolveByFilename(doc.file) ?? formatRegistry.default();

    return (
        <div>
            <DocHeader
                title={doc.title}
                isAdmin={isAdmin}
                onHome={onHome}
                onDownload={onDownload}
            />
            <main className={styles.main}>
                {doc.deprecated && <DeprecatedBanner onRestore={onDeprecate} />}
                <div className="markdown-body">
                    {content && renderer ? (
                        <renderer.component content={content} />
                    ) : (
                        <div style={{ opacity: 0.5 }}>
                            {t("doc.status.loading")}
                        </div>
                    )}
                </div>
                <DocFooter
                    createdAt={doc.created_at}
                    isAdmin={isAdmin}
                    isDeprecated={doc.deprecated}
                    isLocked={doc.locked}
                    onDelete={onDelete}
                    onDeprecate={onDeprecate}
                    onLock={onLock}
                />
            </main>
        </div>
    );
}
