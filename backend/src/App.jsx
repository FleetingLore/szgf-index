import React, { useEffect, useState } from "react"
import Listing from "./pages/Listing"
import DocPage from "./pages/DocPage"
import AdminPage from "./pages/AdminPage"
import { useAdmin } from "./hooks/useAdmin"
import { useCats } from "./hooks/useCats"
import { useDoc } from "./hooks/useDoc"
import { useUpload } from "./hooks/useUpload"
import { downloadDoc } from "./utils/download"

function App() {
  const [view, setView] = useState("loading")
  const { isAdmin, loading: adminLoading, createSession } = useAdmin()
  const { cats, siteName, loading: catsLoading, loadCats } = useCats()
  const { currentDoc, loading: docLoading, fetchDoc, toggleDeprecated, deleteDoc } = useDoc()
  const { upload } = useUpload(loadCats)

  useEffect(() => {
    const path = location.pathname
    if (path === "/" || path === "/listing") {
      setView("listing")
    } else if (path === "/listing/lib/0") {
      setView("admin")
    } else if (path.startsWith("/listing/lib/")) {
      const id = path.split("/").pop()
      fetchDoc(id, () => location.href = "/")
      setView("doc")
    }
  }, [])

  const goHome = () => location.href = "/"
  const goAdmin = () => location.href = "/listing/lib/0"

  if (adminLoading || (view === "listing" && catsLoading)) return <div></div>
  if (view === "admin") return <AdminPage onSuccess={createSession} />
  if (view === "listing") {
    return <Listing siteName={siteName} cats={cats} onUpload={upload} onAdminClick={goAdmin} />
  }
  if (view === "doc" && currentDoc) {
    return <DocPage
      doc={currentDoc}
      isAdmin={isAdmin}
      onDownload={() => downloadDoc(currentDoc)}
      onDeprecate={() => { toggleDeprecated(currentDoc.id); loadCats() }}
      onDelete={() => deleteDoc(currentDoc.id).then(goHome)}
      onHome={goHome}
    />
  }
  if (view === "doc" && docLoading) return <div></div>
  return <div></div>
}

export default App
