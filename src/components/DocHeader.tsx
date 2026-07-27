import React from "react";
import { useStickyHeader } from "../hooks/useStickyHeader";
import styles from "./MenuBar.module.css";

interface DocHeaderProps {
    title: string;
    isAdmin: boolean;
    onHome: () => void;
    onDownload: () => void;
}

export default function DocHeader({
    title,
    isAdmin,
    onHome,
    onDownload,
}: DocHeaderProps) {
    const { headerRef, isSticky, translateY } = useStickyHeader();
    const iconSrc = isAdmin ? "/icons/user.svg" : "/icons/home.svg";
    const iconAlt = isAdmin ? "Admin" : "Home";

    const style: React.CSSProperties = {
        transform: isSticky ? `translateY(${translateY}px)` : "none",
    };

    return (
        <div
            ref={headerRef}
            className={`${styles.bar} ${isSticky ? styles.sticky : ""}`}
            style={style}
        >
            <div className={styles.side}>
                <button className={styles.iconBtn} onClick={onHome}>
                    <img src={iconSrc} alt={iconAlt} width={20} height={20} />
                </button>
            </div>
            <h1 className={styles.title}>{title}</h1>
            <div className={styles.side}>
                <button className={styles.iconBtn} onClick={onDownload}>
                    <img
                        src="/icons/download.svg"
                        alt="Download"
                        width={18}
                        height={18}
                    />
                </button>
            </div>
        </div>
    );
}
