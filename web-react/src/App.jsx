import { useState, useEffect, useRef, useCallback } from 'react'
import './styles.css'

const API_BASE = '/api'

// Get API key from localStorage
const getApiKey = () => localStorage.getItem('digpaper_api_key') || ''
const setApiKey = (key) => localStorage.setItem('digpaper_api_key', key)

// Fetch wrapper that includes API key header
const apiFetch = async (url, options = {}) => {
  const apiKey = getApiKey()
  const headers = {
    ...options.headers,
    'X-API-Key': apiKey
  }
  return fetch(url, { ...options, headers })
}

// Image compression utility - aggressive for fast uploads
const compressImage = async (file, maxWidth = 1280, quality = 0.6) => {
  return new Promise((resolve) => {
    // If not an image, return as-is
    if (!file.type.startsWith('image/')) {
      resolve(file)
      return
    }

    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d')
    const img = new Image()
    
    img.onload = () => {
      let { width, height } = img
      
      // Scale down if too large
      if (width > maxWidth) {
        height = (height * maxWidth) / width
        width = maxWidth
      }
      
      canvas.width = width
      canvas.height = height
      ctx.drawImage(img, 0, 0, width, height)
      
      canvas.toBlob(
        (blob) => {
          if (blob) {
            const compressedFile = new File([blob], file.name, { type: 'image/jpeg' })
            console.log(`Compressed: ${(file.size/1024).toFixed(0)}KB → ${(blob.size/1024).toFixed(0)}KB`)
            resolve(compressedFile)
          } else {
            resolve(file)
          }
        },
        'image/jpeg',
        quality
      )
    }
    
    img.onerror = () => resolve(file)
    img.src = URL.createObjectURL(file)
  })
}

function App() {
  const [tab, setTab] = useState('upload')
  const [documents, setDocuments] = useState([])
  const [projects, setProjects] = useState([])
  const [selectedProject, setSelectedProject] = useState(null)
  const [projectDocs, setProjectDocs] = useState([])
  const [previewDoc, setPreviewDoc] = useState(null)
  const [fullscreen, setFullscreen] = useState(false)
  const [pdfViewerUrl, setPdfViewerUrl] = useState(null)
  const [uploading, setUploading] = useState(false)
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState(null)
  const [showNewProject, setShowNewProject] = useState(false)
  const [newProjectName, setNewProjectName] = useState('')
  const [showSettings, setShowSettings] = useState(false)
  const [apiKeyInput, setApiKeyInput] = useState('')
  const [authenticated, setAuthenticated] = useState(false)
  const [checkingAuth, setCheckingAuth] = useState(true)
  // Pull to refresh state
  const [pullDistance, setPullDistance] = useState(0)
  const [isPulling, setIsPulling] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  // Swipe state
  const [swipedItemId, setSwipedItemId] = useState(null)
  const fileInputRef = useRef(null)
  const contentRef = useRef(null)
  const touchStartY = useRef(0)
  const touchStartX = useRef(0)
  const swipeStartX = useRef(0)

  // Check authentication on load
  useEffect(() => {
    checkAuth()
  }, [])

  const checkAuth = async () => {
    setCheckingAuth(true)
    try {
      const res = await apiFetch(`${API_BASE}/projects`)
      if (res.ok) {
        setAuthenticated(true)
        const data = await res.json()
        setProjects(data)
      } else if (res.status === 401) {
        setAuthenticated(false)
        setShowSettings(true)
        setApiKeyInput(getApiKey())
      }
    } catch (e) {
      // Network error - might still be authenticated
      setAuthenticated(false)
      setShowSettings(true)
    }
    setCheckingAuth(false)
  }

  const saveApiKey = async () => {
    setApiKey(apiKeyInput)
    setShowSettings(false)
    await checkAuth()
  }

  useEffect(() => {
    if (tab === 'inbox') loadInbox()
    if (tab === 'projects') loadProjects()
  }, [tab])

  const loadInbox = async () => {
    setLoading(true)
    try {
      const res = await apiFetch(`${API_BASE}/documents/inbox`)
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      const data = await res.json()
      setDocuments(data)
    } catch (e) {
      showMessage('Erro ao carregar documentos', 'error')
    }
    setLoading(false)
  }

  const loadProjects = async () => {
    setLoading(true)
    try {
      const res = await apiFetch(`${API_BASE}/projects`)
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      const data = await res.json()
      setProjects(data)
    } catch (e) {
      showMessage('Erro ao carregar obras', 'error')
    }
    setLoading(false)
  }

  const loadProjectDocs = async (projectId) => {
    setLoading(true)
    try {
      const res = await apiFetch(`${API_BASE}/projects/${projectId}/documents`)
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      const data = await res.json()
      setProjectDocs(data)
    } catch (e) {
      showMessage('Erro ao carregar documentos', 'error')
    }
    setLoading(false)
  }

  const showMessage = (text, type = 'success') => {
    setMessage({ text, type })
    setTimeout(() => setMessage(null), 3000)
  }

  const handleFileSelect = async (e) => {
    const file = e.target.files?.[0]
    if (!file) return

    setUploading(true)
    
    try {
      // Compress image before upload
      const compressedFile = await compressImage(file)
      const formData = new FormData()
      formData.append('file', compressedFile)

      const res = await apiFetch(`${API_BASE}/upload`, {
        method: 'POST',
        body: formData
      })
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      if (res.ok) {
        showMessage('Foto enviada com sucesso!')
        fileInputRef.current.value = ''
      } else {
        showMessage('Erro ao enviar foto', 'error')
      }
    } catch (e) {
      showMessage('Erro de conexão', 'error')
    }
    setUploading(false)
  }

  const assignToProject = async (docId, projectId) => {
    try {
      const res = await apiFetch(`${API_BASE}/documents/${docId}/assign`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project_id: projectId })
      })
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      if (res.ok) {
        showMessage('Documento movido!')
        loadInbox()
        loadProjects()
        setPreviewDoc(null)
      }
    } catch (e) {
      showMessage('Erro ao mover documento', 'error')
    }
  }

  const deleteDocument = async (docId) => {
    if (!confirm('Tem certeza que deseja excluir este documento?')) return
    
    try {
      const res = await apiFetch(`${API_BASE}/documents/${docId}`, {
        method: 'DELETE'
      })
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      if (res.ok) {
        showMessage('Documento excluído!')
        setPreviewDoc(null)
        loadInbox()
        loadProjects()
      } else {
        showMessage('Erro ao excluir documento', 'error')
      }
    } catch (e) {
      showMessage('Erro ao excluir documento', 'error')
    }
  }

  const createProject = async () => {
    if (!newProjectName.trim()) return
    try {
      const res = await apiFetch(`${API_BASE}/projects`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newProjectName.trim() })
      })
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      if (res.ok) {
        showMessage('Obra criada com sucesso!')
        setNewProjectName('')
        setShowNewProject(false)
        loadProjects()
      }
    } catch (e) {
      showMessage('Erro ao criar obra', 'error')
    }
  }

  const toggleProjectStatus = async (project) => {
    const newStatus = project.status === 'ACTIVE' ? 'ARCHIVED' : 'ACTIVE'
    try {
      const res = await apiFetch(`${API_BASE}/projects/${project.id}/status`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status: newStatus })
      })
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true); return }
      if (res.ok) {
        showMessage(newStatus === 'ARCHIVED' ? 'Obra arquivada' : 'Obra reativada')
        loadProjects()
        if (selectedProject?.id === project.id) {
          setSelectedProject({ ...project, status: newStatus })
        }
      }
    } catch (e) {
      showMessage('Erro ao atualizar obra', 'error')
    }
  }

  const openProject = (project) => {
    setSelectedProject(project)
    loadProjectDocs(project.id)
  }

  // Use file_type field for reliable file type detection
  const isImage = (doc) => {
    return doc.file_type === 'image'
  }

  const isPdf = (doc) => {
    return doc.file_type === 'pdf' || doc.original_name?.toLowerCase().endsWith('.pdf')
  }

  const openPdf = (doc) => {
    setPdfViewerUrl(doc.file_url)
  }

  const triggerFileInput = (useCamera = true) => {
    if (useCamera) {
      fileInputRef.current.setAttribute('capture', 'environment')
    } else {
      fileInputRef.current.removeAttribute('capture')
    }
    fileInputRef.current?.click()
  }

  const formatDate = (dateStr) => {
    if (!dateStr) return ''
    const d = new Date(dateStr)
    return d.toLocaleDateString('pt-BR')
  }

  const getPageTitle = () => {
    if (tab === 'upload') return 'Fotografar'
    if (tab === 'inbox') return 'Caixa de Entrada'
    if (tab === 'projects') {
      if (selectedProject) return selectedProject.name
      return 'Obras'
    }
    return 'DigPaper'
  }

  const handleRefresh = async () => {
    if (tab === 'inbox') await loadInbox()
    else if (tab === 'projects' && !selectedProject) await loadProjects()
    else if (selectedProject) await loadProjectDocs(selectedProject.id)
  }

  // Pull to refresh handlers
  const handleTouchStart = useCallback((e) => {
    const content = contentRef.current
    if (!content || content.scrollTop > 5) return
    touchStartY.current = e.touches[0].clientY
    setIsPulling(true)
  }, [])

  const handleTouchMove = useCallback((e) => {
    if (!isPulling || isRefreshing) return
    const content = contentRef.current
    if (!content || content.scrollTop > 0) {
      setPullDistance(0)
      return
    }
    
    const touchY = e.touches[0].clientY
    const distance = Math.max(0, (touchY - touchStartY.current) * 0.5)
    setPullDistance(Math.min(distance, 80))
  }, [isPulling, isRefreshing])

  const handleTouchEnd = useCallback(async () => {
    if (!isPulling) return
    setIsPulling(false)
    
    if (pullDistance >= 60) {
      setIsRefreshing(true)
      await handleRefresh()
      setIsRefreshing(false)
    }
    setPullDistance(0)
  }, [isPulling, pullDistance])

  // Swipe handlers for document cards
  const handleSwipeStart = (e, itemId) => {
    swipeStartX.current = e.touches[0].clientX
    touchStartX.current = e.touches[0].clientX
  }

  const handleSwipeMove = (e, itemId) => {
    const deltaX = e.touches[0].clientX - swipeStartX.current
    if (Math.abs(deltaX) > 50) {
      setSwipedItemId(deltaX < 0 ? itemId : null)
    }
  }

  const handleSwipeEnd = () => {
    // Keep swiped state for action buttons
  }

  // Reset swipe when clicking elsewhere
  const resetSwipe = () => setSwipedItemId(null)

  // Show loading while checking auth
  if (checkingAuth) {
    return (
      <div className="app">
        <div className="auth-screen">
          <div className="spinner"></div>
          <p>Verificando...</p>
        </div>
      </div>
    )
  }

  // Show settings/login screen if not authenticated
  if (showSettings || !authenticated) {
    return (
      <div className="app">
        <div className="auth-screen">
          <div className="auth-card">
            <h2>Chave de Acesso</h2>
            <p>Digite a chave de acesso para entrar no sistema.</p>
            <input
              type="password"
              value={apiKeyInput}
              onChange={(e) => setApiKeyInput(e.target.value)}
              placeholder="Chave de acesso"
              className="auth-input"
              autoFocus
              onKeyDown={(e) => e.key === 'Enter' && saveApiKey()}
            />
            <button className="btn-primary" onClick={saveApiKey}>
              Entrar
            </button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="app">
      <header className="header">
        {selectedProject && (
          <button className="header-back" onClick={() => { setSelectedProject(null); setProjectDocs([]) }}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M19 12H5M12 19l-7-7 7-7"/>
            </svg>
          </button>
        )}
        <h1>{getPageTitle()}</h1>
        <div className="header-actions">
          {selectedProject && (
            <button 
              className="header-action" 
              onClick={() => toggleProjectStatus(selectedProject)}
              title={selectedProject.status === 'ACTIVE' ? 'Arquivar obra' : 'Reativar obra'}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                {selectedProject.status === 'ACTIVE' ? (
                  <path d="M21 8v13H3V8M1 3h22v5H1zM10 12h4"/>
                ) : (
                  <path d="M21 8v13H3V8M1 3h22v5H1zM12 12v6M9 15l3 3 3-3"/>
                )}
              </svg>
            </button>
          )}
          {(tab === 'inbox' || tab === 'projects') && (
            <button className={`header-action ${loading ? 'spinning' : ''}`} onClick={handleRefresh} disabled={loading}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M23 4v6h-6M1 20v-6h6"/>
                <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
              </svg>
            </button>
          )}
          <button className="header-action" onClick={() => { setApiKeyInput(getApiKey()); setShowSettings(true) }}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="12" cy="12" r="3"/>
              <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
            </svg>
          </button>
        </div>
      </header>

      {message && (
        <div className={`toast ${message.type}`}>{message.text}</div>
      )}

      {/* Pull to refresh indicator */}
      {(pullDistance > 0 || isRefreshing) && (
        <div 
          className="pull-indicator" 
          style={{ 
            height: isRefreshing ? 50 : pullDistance,
            opacity: isRefreshing ? 1 : pullDistance / 60
          }}
        >
          <div className={`pull-spinner ${isRefreshing ? 'spinning' : ''}`}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M23 4v6h-6M1 20v-6h6"/>
              <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
            </svg>
          </div>
        </div>
      )}

      <main 
        className="content" 
        ref={contentRef}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onClick={resetSwipe}
      >
        {/* FOTOGRAFAR TAB */}
        {tab === 'upload' && (
          <div className="upload-section">
            <div className="scan-icon">
              <svg viewBox="0 0 80 80" fill="none">
                {/* Corner brackets */}
                <path d="M4 20V8a4 4 0 014-4h12" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
                <path d="M60 4h12a4 4 0 014 4v12" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
                <path d="M76 60v12a4 4 0 01-4 4H60" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
                <path d="M20 76H8a4 4 0 01-4-4V60" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
                {/* Document icon */}
                <rect x="24" y="20" width="32" height="40" rx="3" stroke="currentColor" strokeWidth="2.5"/>
                <path d="M32 32h16M32 40h16M32 48h10" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round"/>
              </svg>
            </div>
            <h2>Adicionar Documento</h2>
            <p>Fotografe um esboço ou adicione um PDF.<br/>O documento irá para a Caixa de Entrada.</p>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*,application/pdf"
              onChange={handleFileSelect}
              hidden
            />
            <div className="upload-buttons">
              <button 
                className="btn-primary"
                onClick={() => triggerFileInput(true)}
                disabled={uploading}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2v11z"/>
                  <circle cx="12" cy="13" r="4"/>
                </svg>
                {uploading ? 'Enviando...' : 'Tirar Foto'}
              </button>
              <button 
                className="btn-secondary"
                onClick={() => triggerFileInput(false)}
                disabled={uploading}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                  <circle cx="8.5" cy="8.5" r="1.5"/>
                  <path d="M21 15l-5-5L5 21"/>
                </svg>
                Escolher Imagem
              </button>
              <button 
                className="btn-secondary btn-pdf"
                onClick={() => { fileInputRef.current.removeAttribute('capture'); fileInputRef.current.accept = 'application/pdf'; fileInputRef.current.click(); fileInputRef.current.accept = 'image/*,application/pdf'; }}
                disabled={uploading}
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                  <path d="M14 2v6h6"/>
                  <path d="M10 12h4M10 16h4M10 20h4" strokeWidth="1.5"/>
                </svg>
                Adicionar PDF
              </button>
            </div>
          </div>
        )}

        {/* CAIXA DE ENTRADA TAB */}
        {tab === 'inbox' && !previewDoc && (
          <div className="section">
            {loading && documents.length === 0 ? (
              <div className="doc-grid">
                {/* Skeleton loading */}
                {[1, 2, 3, 4, 5, 6].map(i => (
                  <div key={i} className="doc-card skeleton">
                    <div className="doc-thumb skeleton-thumb"></div>
                    <div className="doc-info">
                      <span className="skeleton-text"></span>
                      <span className="skeleton-text short"></span>
                    </div>
                  </div>
                ))}
              </div>
            ) : documents.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M22 12h-6l-2 3h-4l-2-3H2"/>
                    <path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z"/>
                  </svg>
                </div>
                <h3>Caixa de Entrada Vazia</h3>
                <p>Os documentos fotografados<br/>aparecerão aqui.</p>
              </div>
            ) : (
              <div className="doc-grid">
                {documents.map(doc => (
                  <div 
                    key={doc.id} 
                    className={`doc-card-wrapper ${swipedItemId === doc.id ? 'swiped' : ''}`}
                    onTouchStart={(e) => handleSwipeStart(e, doc.id)}
                    onTouchMove={(e) => handleSwipeMove(e, doc.id)}
                    onTouchEnd={handleSwipeEnd}
                  >
                    <div className="doc-card touchable" onClick={() => isPdf(doc) ? openPdf(doc) : setPreviewDoc(doc)}>
                      <div className={`doc-thumb ${isPdf(doc) ? 'pdf-thumb' : ''}`}>
                        {isImage(doc) ? (
                          <img src={doc.file_url} alt={doc.original_name} loading="lazy" />
                        ) : isPdf(doc) ? (
                          <div className="pdf-badge">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                              <path d="M14 2v6h6"/>
                            </svg>
                            <span>PDF</span>
                          </div>
                        ) : (
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                            <path d="M14 2v6h6M16 13H8M16 17H8M10 9H8"/>
                          </svg>
                        )}
                      </div>
                      <div className="doc-info">
                        <span className="doc-name">{doc.original_name}</span>
                        <span className="doc-date">{formatDate(doc.uploaded_at)}</span>
                      </div>
                    </div>
                    <div className="swipe-actions">
                      <button className="swipe-delete" onClick={(e) => { e.stopPropagation(); deleteDocument(doc.id) }}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                        </svg>
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {tab === 'inbox' && previewDoc && (
          <div className="preview-section">
            <div className="preview-header">
              <button className="btn-back" onClick={() => setPreviewDoc(null)}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M19 12H5M12 19l-7-7 7-7"/>
                </svg>
                Voltar
              </button>
              <button className="btn-delete" onClick={() => deleteDocument(previewDoc.id)}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                  <line x1="10" y1="11" x2="10" y2="17"/>
                  <line x1="14" y1="11" x2="14" y2="17"/>
                </svg>
              </button>
            </div>
            <div 
              className="preview-image" 
              onClick={() => {
                if (isImage(previewDoc)) setFullscreen(true)
                else if (isPdf(previewDoc)) openPdf(previewDoc)
              }} 
              style={(isImage(previewDoc) || isPdf(previewDoc)) ? {cursor: 'pointer'} : {}}
            >
              {isImage(previewDoc) ? (
                <img src={previewDoc.file_url} alt={previewDoc.original_name} />
              ) : isPdf(previewDoc) ? (
                <div className="doc-icon-large pdf-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                    <path d="M14 2v6h6"/>
                  </svg>
                  <span className="pdf-label">PDF</span>
                  <span className="pdf-tap">Toque para abrir</span>
                </div>
              ) : (
                <div className="doc-icon-large">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                    <path d="M14 2v6h6"/>
                  </svg>
                </div>
              )}
            </div>
            <div className="preview-info">
              <h3>{previewDoc.original_name}</h3>
              <p>{formatDate(previewDoc.uploaded_at)}</p>
            </div>
            <div className="assign-section">
              <h4>Mover para Obra</h4>
              {projects.filter(p => p.status === 'ACTIVE').length === 0 ? (
                <p className="text-muted">Nenhuma obra ativa</p>
              ) : (
                <div className="project-list">
                  {projects.filter(p => p.status === 'ACTIVE').map(p => (
                    <button 
                      key={p.id} 
                      className="btn-project"
                      onClick={() => assignToProject(previewDoc.id, p.id)}
                    >
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                        <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2v11z"/>
                      </svg>
                      {p.name}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        {/* OBRAS TAB */}
        {tab === 'projects' && !selectedProject && (
          <div className="section">
            {loading && projects.length === 0 ? (
              <div className="project-list-view">
                {/* Skeleton loading for projects */}
                {[1, 2, 3, 4].map(i => (
                  <div key={i} className="project-row skeleton">
                    <div className="project-folder-icon skeleton-icon"></div>
                    <div className="project-details">
                      <span className="skeleton-text"></span>
                      <span className="skeleton-text short"></span>
                    </div>
                  </div>
                ))}
              </div>
            ) : projects.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2v11z"/>
                  </svg>
                </div>
                <h3>Nenhuma Obra</h3>
                <p>Crie uma nova obra para<br/>organizar seus documentos.</p>
              </div>
            ) : (
              <div className="project-list-view">
                {projects.map(p => (
                  <div 
                    key={p.id} 
                    className={`project-row touchable ${p.status === 'ARCHIVED' ? 'archived' : ''}`}
                    onClick={() => openProject(p)}
                  >
                    <div className="project-folder-icon">
                      <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M20 6h-8l-2-2H4a2 2 0 00-2 2v12a2 2 0 002 2h16a2 2 0 002-2V8a2 2 0 00-2-2z"/>
                      </svg>
                    </div>
                    <div className="project-details">
                      <span className="project-name">{p.name}</span>
                      <div className="project-meta">
                        <span className={`status-badge ${p.status === 'ACTIVE' ? 'active' : 'archived'}`}>
                          {p.status === 'ACTIVE' ? 'ATIVA' : 'ARQUIVADA'}
                        </span>
                        <span className="project-date">{formatDate(p.created_at)}</span>
                      </div>
                    </div>
                    <div className="project-chevron">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M9 18l6-6-6-6"/>
                      </svg>
                    </div>
                  </div>
                ))}
              </div>
            )}
            
            {/* FAB */}
            <button className="fab" onClick={() => setShowNewProject(true)}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 5v14M5 12h14"/>
              </svg>
              Nova Obra
            </button>
          </div>
        )}

        {tab === 'projects' && selectedProject && (
          <div className="section">
            {loading && projectDocs.length === 0 ? (
              <div className="loading-state">
                <div className="spinner"></div>
                <p>Carregando...</p>
              </div>
            ) : projectDocs.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                    <path d="M14 2v6h6"/>
                  </svg>
                </div>
                <h3>Nenhum Documento</h3>
                <p>Esta obra ainda não tem<br/>documentos associados.</p>
              </div>
            ) : (
              <div className="doc-grid">
                {projectDocs.map(doc => (
                  <div 
                    key={doc.id} 
                    className="doc-card touchable" 
                    onClick={() => isPdf(doc) ? openPdf(doc) : (isImage(doc) ? window.open(doc.file_url, '_blank') : window.open(doc.file_url, '_blank'))}
                  >
                    <div className={`doc-thumb ${isPdf(doc) ? 'pdf-thumb' : ''}`}>
                      {isImage(doc) ? (
                        <img src={doc.file_url} alt={doc.original_name} loading="lazy" />
                      ) : isPdf(doc) ? (
                        <div className="pdf-badge">
                          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                            <path d="M14 2v6h6"/>
                          </svg>
                          <span>PDF</span>
                        </div>
                      ) : (
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
                          <path d="M14 2v6h6"/>
                        </svg>
                      )}
                    </div>
                    <div className="doc-info">
                      <span className="doc-name">{doc.original_name}</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </main>

      {/* New Project Modal */}
      {showNewProject && (
        <div className="modal-overlay" onClick={() => setShowNewProject(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <h3>Nova Obra</h3>
            <input
              type="text"
              placeholder="Nome da obra"
              value={newProjectName}
              onChange={e => setNewProjectName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && createProject()}
              autoFocus
            />
            <div className="modal-actions">
              <button className="btn-cancel" onClick={() => setShowNewProject(false)}>Cancelar</button>
              <button className="btn-confirm" onClick={createProject}>Criar</button>
            </div>
          </div>
        </div>
      )}

      {/* Bottom Navigation */}
      <nav className="bottom-nav">
        <button className={tab === 'upload' ? 'active' : ''} onClick={() => setTab('upload')}>
          <div className="nav-icon-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M23 19a2 2 0 01-2 2H3a2 2 0 01-2-2V8a2 2 0 012-2h4l2-3h6l2 3h4a2 2 0 012 2v11z"/>
              <circle cx="12" cy="13" r="4"/>
              <path d="M12 10v6M9 13h6" strokeWidth="1.5"/>
            </svg>
          </div>
          <span>Fotografar</span>
        </button>
        <button className={tab === 'inbox' ? 'active' : ''} onClick={() => { setTab('inbox'); setPreviewDoc(null) }}>
          <div className="nav-icon-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 12h-6l-2 3h-4l-2-3H2"/>
              <path d="M5.45 5.11L2 12v6a2 2 0 002 2h16a2 2 0 002-2v-6l-3.45-6.89A2 2 0 0016.76 4H7.24a2 2 0 00-1.79 1.11z"/>
            </svg>
          </div>
          <span>Caixa de Entrada</span>
        </button>
        <button className={tab === 'projects' ? 'active' : ''} onClick={() => { setTab('projects'); setSelectedProject(null) }}>
          <div className="nav-icon-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2v11z"/>
            </svg>
          </div>
          <span>Obras</span>
        </button>
      </nav>

      {/* Fullscreen Image Overlay */}
      {fullscreen && previewDoc && isImage(previewDoc) && (
        <div className="fullscreen-overlay" onClick={() => setFullscreen(false)}>
          <button className="fullscreen-close" onClick={() => setFullscreen(false)}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
          <img src={previewDoc.file_url} alt={previewDoc.original_name} onClick={(e) => e.stopPropagation()} />
        </div>
      )}

      {/* PDF Viewer Overlay */}
      {pdfViewerUrl && (
        <div className="pdf-viewer-overlay">
          <div className="pdf-viewer-header">
            <button className="pdf-viewer-close" onClick={() => setPdfViewerUrl(null)}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M19 12H5M12 19l-7-7 7-7"/>
              </svg>
              Voltar
            </button>
          </div>
          <iframe 
            src={pdfViewerUrl} 
            className="pdf-viewer-frame"
            title="PDF Viewer"
          />
        </div>
      )}
    </div>
  )
}

export default App
