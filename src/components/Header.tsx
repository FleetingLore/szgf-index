import React from "react";
import { useStickyHeader } from "../hooks/useStickyHeader";
import styles from "./MenuBar.module.css";

interface HeaderProps {
    siteName: string;
    isAdmin: boolean;
    onAdminClick: () => void;
    onUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

export default function Header({
    siteName,
    isAdmin,
    onAdminClick,
    onUpload,
}: HeaderProps) {
    const { headerRef, isSticky, translateY } = useStickyHeader();
    const iconSrc = isAdmin ? "/icons/user.svg" : "/icons/home.svg";
    const iconAlt = isAdmin ? "Logout" : "Admin";

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
                <button className={styles.iconBtn} onClick={onAdminClick}>
                    <img src={iconSrc} alt={iconAlt} width={20} height={20} />
                </button>
            </div>
            <h1 className={styles.title}>{siteName}</h1>
            <div className={styles.side}>
                <label className={styles.iconBtn} style={{ cursor: "pointer" }}>
                    <img
                        src="/icons/upload.svg"
                        alt="Upload"
                        width={18}
                        height={18}
                    />
                    <input
                        type="file"
                        accept=".md"
                        onChange={onUpload}
                        style={{ display: "none" }}
                    />
                </label>
            </div>
        </div>
    );
}
