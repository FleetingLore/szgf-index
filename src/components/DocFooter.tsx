import React, { useState } from "react";
import { useI18n } from "../hooks/useI18n";
import { debugLog } from "../utils/debug";
import styles from "./DocFooter.module.css";

interface DocFooterProps {
    createdAt: number;
    isAdmin: boolean;
    isDeprecated: boolean;
    isLocked: boolean;
    onDelete: () => void;
    onDeprecate: () => void;
    onLock: () => void;
}

export default function DocFooter({
    createdAt,
    isAdmin,
    isDeprecated,
    isLocked,
    onDelete,
    onDeprecate,
    onLock,
}: DocFooterProps) {
    const { t } = useI18n();
    const [confirm, setConfirm] = useState(false);

    const handleDeleteClick = () => {
        if (confirm) {
            onDelete();
            setConfirm(false);
        } else {
            setConfirm(true);
            setTimeout(() => setConfirm(false), 3000);
        }
    };

    const lockedForVisitor = isLocked && !isAdmin;
    const canDeprecate = !isDeprecated && !lockedForVisitor;
    const canRestore = isDeprecated && !isLocked;
    const canDelete = isDeprecated && isAdmin;

    debugLog(
        "DocFooter",
        "isAdmin:",
        isAdmin,
        "isDeprecated:",
        isDeprecated,
        "isLocked:",
        isLocked,
        "lockedForVisitor:",
        lockedForVisitor,
        "canDeprecate:",
        canDeprecate,
        "canRestore:",
        canRestore,
        "canDelete:",
        canDelete,
    );

    return (
        <div className={styles.footer}>
            {/* 锁定按钮（仅管理员可见） */}
            {isAdmin && (
                <button className={styles.actionLock} onClick={onLock}>
                    {isLocked ? t("doc.action.unlock") : t("doc.action.lock")}
                </button>
            )}

            {/* 访客看到的锁定提示 */}
            {lockedForVisitor && !isDeprecated && (
                <span className={styles.lockedLabel}>
                    {t("doc.status.locked")}
                </span>
            )}

            {/* 弃用/恢复按钮 */}
            {canDeprecate && (
                <button className={styles.action} onClick={onDeprecate}>
                    {t("doc.action.mark_deprecated")}
                </button>
            )}
            {canRestore && (
                <button className={styles.action} onClick={onDeprecate}>
                    {t("doc.action.restore")}
                </button>
            )}

            {/* 删除按钮 */}
            {canDelete && (
                <button className={styles.action} onClick={handleDeleteClick}>
                    {confirm
                        ? t("doc.action.confirm_delete")
                        : t("doc.action.delete")}
                </button>
            )}

            {/* 占位 */}
            {isDeprecated && !canRestore && !canDelete && <div />}
            <div className={styles.date}>
                {new Date(createdAt).toLocaleString()}
            </div>
        </div>
    );
}
