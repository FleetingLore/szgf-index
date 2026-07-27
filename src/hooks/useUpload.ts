import { useCallback } from "react";
import { useI18n } from "./useI18n";
import { formatRegistry } from "../rendering/registry";

interface UseUploadReturn {
    upload: (e: React.ChangeEvent<HTMLInputElement>) => Promise<void>;
}

export function useUpload(loadCats: () => Promise<void>): UseUploadReturn {
    const { t } = useI18n();

    const upload = useCallback(
        async (e: React.ChangeEvent<HTMLInputElement>) => {
            const file = e.target.files?.[0];
            if (!file) return;

            // 通过格式注册表检查文件类型
            const ext = file.name.split(".").pop()?.toLowerCase();
            const supported = formatRegistry.allExtensions();
            if (!ext || !supported.includes(ext)) {
                alert(t("doc.upload.only_md"));
                return;
            }

            const reader = new FileReader();
            reader.onload = async (ev) => {
                const content = ev.target?.result as string;
                const title = file.name.slice(0, -(ext.length + 1));
                await fetch("/api/cats", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ title, content }),
                });
                loadCats();
            };
            reader.readAsText(file);
            e.target.value = "";
        },
        [loadCats, t],
    );

    return { upload };
}
