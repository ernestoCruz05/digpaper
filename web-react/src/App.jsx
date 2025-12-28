import { useState, useEffect, useRef } from 'react'
import './styles.css'

const API_BASE = '/api'

function App() {
  const [tab, setTab] = useState('upload')
  const [documents, setDocuments] = useState([])
  const [projects, setProjects] = useState([])
  const [selectedProject, setSelectedProject] = useState(null)
  const [projectDocs, setProjectDocs] = useState([])
  const [previewDoc, setPreviewDoc] = useState(null)
  const [fullscreen, setFullscreen] = useState(false)
  const [uploading, setUploading] = useState(false)
  const [loading, setLoading] = useState(false)
  const [message, setMessage] = useState(null)
  const [showNewProject, setShowNewProject] = useState(false)
  const [newProjectName, setNewProjectName] = useState('')
  const fileInputRef = useRef(null)

  useEffect(() => {
    loadProjects()
  }, [])

  useEffect(() => {
    if (tab === 'inbox') loadInbox()
    if (tab === 'projects') loadProjects()
  }, [tab])

  const loadInbox = async () => {
    setLoading(true)
    try {
      const res = await fetch(`${API_BASE}/documents/inbox`)
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
      const res = await fetch(`${API_BASE}/projects`)
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
      const res = await fetch(`${API_BASE}/projects/${projectId}/documents`)
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
    const formData = new FormData()
    formData.append('file', file)

    try {
      const res = await fetch(`${API_BASE}/upload`, {
        method: 'POST',
        body: formData
      })
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
      const res = await fetch(`${API_BASE}/documents/${docId}/assign`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ project_id: projectId })
      })
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
      const res = await fetch(`${API_BASE}/documents/${docId}`, {
        method: 'DELETE'
      })
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
      const res = await fetch(`${API_BASE}/projects`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newProjectName.trim() })
      })
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

  const openProject = (project) => {
    setSelectedProject(project)
    loadProjectDocs(project.id)
  }

  // Use file_type field for reliable image detection
  const isImage = (doc) => {
    return doc.file_type === 'image'
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
    if (tab === 'upload') return 'Fotografar Documento'
    if (tab === 'inbox') return 'Caixa de Entrada'
    if (tab === 'projects') {
      if (selectedProject) return selectedProject.name
      return 'Obras'
    }
    return 'DigPaper'
  }

  const handleRefresh = () => {
    if (tab === 'inbox') loadInbox()
    else if (tab === 'projects' && !selectedProject) loadProjects()
    else if (selectedProject) loadProjectDocs(selectedProject.id)
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
        {(tab === 'inbox' || tab === 'projects') && (
          <button className={`header-action ${loading ? 'spinning' : ''}`} onClick={handleRefresh} disabled={loading}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M23 4v6h-6M1 20v-6h6"/>
              <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
            </svg>
          </button>
        )}
      </header>

      {message && (
        <div className={`toast ${message.type}`}>{message.text}</div>
      )}

      <main className="content">
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
            <h2>Fotografar Esboço ou Lista</h2>
            <p>Tire uma foto do documento.<br/>A foto será enviada para a Caixa de Entrada.</p>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*"
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
                Escolher da Galeria
              </button>
            </div>
          </div>
        )}

        {/* CAIXA DE ENTRADA TAB */}
        {tab === 'inbox' && !previewDoc && (
          <div className="section">
            {loading && documents.length === 0 ? (
              <div className="loading-state">
                <div className="spinner"></div>
                <p>Carregando...</p>
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
                  <div key={doc.id} className="doc-card" onClick={() => setPreviewDoc(doc)}>
                    <div className="doc-thumb">
                      {isImage(doc) ? (
                        <img src={doc.file_url} alt={doc.original_name} loading="lazy" />
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
            <div className="preview-image" onClick={() => isImage(previewDoc) && setFullscreen(true)} style={isImage(previewDoc) ? {cursor: 'pointer'} : {}}>
              {isImage(previewDoc) ? (
                <img src={previewDoc.file_url} alt={previewDoc.original_name} />
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
              <div className="loading-state">
                <div className="spinner"></div>
                <p>Carregando...</p>
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
                    className={`project-row ${p.status === 'ARCHIVED' ? 'archived' : ''}`}
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
                  <div key={doc.id} className="doc-card" onClick={() => window.open(doc.file_url, '_blank')}>
                    <div className="doc-thumb">
                      {isImage(doc) ? (
                        <img src={doc.file_url} alt={doc.original_name} loading="lazy" />
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
    </div>
  )
}

export default App
