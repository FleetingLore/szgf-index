import React from "react";
import { useI18n } from "../hooks/useI18n";
import styles from "./DeprecatedBanner.module.css";

interface DeprecatedBannerProps {
    onRestore: () => void;
}

export default function DeprecatedBanner({ onRestore }: DeprecatedBannerProps) {
    const { t } = useI18n();

    return (
        <div className={styles.banner}>
            <span className={styles.text}>{t("doc.status.deprecated")}</span>
            <button className={styles.button} onClick={onRestore}>
                {t("doc.action.restore")}
            </button>
        </div>
    );
}
