import React, { useState, useEffect } from "react";
import { useI18n } from "../hooks/useI18n";
import type { SequenceResponse, CreateSessionResponse } from "../types/api";
import barStyles from "../components/MenuBar.module.css";
import styles from "./AdminPage.module.css";

interface AdminPageProps {
    onSuccess: (token: string) => void;
}

export default function AdminPage({ onSuccess }: AdminPageProps) {
    const { t } = useI18n();
    const [sequence, setSequence] = useState<number[]>([]);
    const [userInput, setUserInput] = useState<number[]>([]);
    const [clickCounts, setClickCounts] = useState<Record<number, number>>({});
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        fetch("/api/admin/sequence")
            .then((res) => res.json())
            .then((data: SequenceResponse) => {
                if (data.sequence?.length) setSequence(data.sequence);
                setLoading(false);
            })
            .catch(() => setLoading(false));
    }, []);

    const handleClick = async (num: number) => {
        const newCount = (clickCounts[num] || 0) + 1;
        setClickCounts((prev) => ({ ...prev, [num]: newCount }));

        if (!sequence.length) return;

        const newInput = [...userInput, num];
        setUserInput(newInput);

        const seqStr = sequence.join(",");
        const inputStr = newInput.join(",");

        if (inputStr.endsWith(seqStr)) {
            const res = await fetch("/api/admin/session", { method: "POST" });
            const data: CreateSessionResponse = await res.json();
            if (data.token) {
                onSuccess(data.token);
            }
        } else if (!seqStr.startsWith(inputStr)) {
            setUserInput([]);
        }
    };

    if (loading) return <div>{t("doc.status.loading")}</div>;

    const getStyle = (num: number): React.CSSProperties => {
        const count = clickCounts[num] || 0;
        if (count === 0) {
            return {
                backgroundColor: "rgba(31, 35, 40, 0.3)",
                border: "2px solid #1f2328",
            };
        }
        if (count % 2 === 1) {
            return {
                backgroundColor: "rgba(144, 223, 223, 0.3)",
                border: "2px solid #90DFDF",
            };
        }
        return {
            backgroundColor: "rgba(31, 35, 40, 0.3)",
            border: "2px solid #1f2328",
        };
    };

    return (
        <div>
            <div className={barStyles.bar}>
                <div className={barStyles.side}>
                    <button
                        className={barStyles.iconBtn}
                        onClick={() => {
                            window.location.href = "/";
                        }}
                    >
                        <img
                            src="/icons/home.svg"
                            alt={t("admin.button_home")}
                            width={20}
                            height={20}
                        />
                    </button>
                </div>
                <h1 className={barStyles.title}>{t("admin.title")}</h1>
                <div className={barStyles.side}>
                    <div style={{ width: "40px" }} />
                </div>
            </div>
            <div className={styles.wrapper}>
                <div className={styles.grid}>
                    {[2, 1, 3, 4].map((pos) => (
                        <button
                            key={pos}
                            onClick={() => handleClick(pos)}
                            style={getStyle(pos)}
                            className={styles.block}
                        />
                    ))}
                </div>
            </div>
        </div>
    );
}
