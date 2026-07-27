import React from "react";
import { useI18n } from "../hooks/useI18n";
import Header from "../components/Header";
import DocItem from "../components/DocItem";
import type { DocListItem } from "../types";
import styles from "./Listing.module.css";

interface ListingProps {
    siteName: string;
    cats: DocListItem[];
    isAdmin: boolean;
    onUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
    onAdminClick: () => void;
}

export default function Listing({
    siteName,
    cats,
    isAdmin,
    onUpload,
    onAdminClick,
}: ListingProps) {
    const { t } = useI18n();

    return (
        <div>
            <Header
                siteName={siteName}
                isAdmin={isAdmin}
                onAdminClick={onAdminClick}
                onUpload={onUpload}
            />
            <main className={styles.main}>
                {!cats.length && (
                    <div className={styles.empty}>{t("doc.status.empty")}</div>
                )}
                {cats.map((cat) => (
                    <DocItem
                        key={cat.id}
                        cat={cat}
                        onClick={() => {
                            location.href = `/listing/lib/${cat.id}`;
                        }}
                    />
                ))}
            </main>
        </div>
    );
}
