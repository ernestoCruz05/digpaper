import { useState, useEffect, useRef } from 'react'
import imageCompression from 'browser-image-compression'
import { saveUploadToQueue, getUploadQueue, removeFromQueue, cacheMessages, getCachedMessages } from './db'
import './styles.css'

const API_BASE = '/api'

const getApiKey = () => localStorage.getItem('digpaper_api_key') || ''
const setApiKeyStore = (key) => localStorage.setItem('digpaper_api_key', key)
const getAuthorName = () => localStorage.getItem('digpaper_author') || ''
const setAuthorNameStore = (name) => localStorage.setItem('digpaper_author', name)

const apiFetch = (url, options = {}) => {
  const headers = options.headers || {}
  const apiKey = getApiKey()
  if (apiKey) headers['X-API-Key'] = apiKey
  return fetch(url, { ...options, headers })
}

const compressImage = async (file, maxWidth = 1600, maxSizeMB = 0.5) => {
  if (!file.type.startsWith('image/')) return file
  try {
    return await imageCompression(file, {
      maxSizeMB, maxWidthOrHeight: maxWidth, useWebWorker: true,
      initialQuality: 0.7, fileType: 'image/jpeg'
    })
  } catch { return file }
}

// ── SVG Icons ──
const IconCamera = () => <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/></svg>
const IconUpload = () => <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
const IconArchive = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/></svg>
const IconReactivate = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
const IconMic = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="2" width="6" height="12" rx="3"/><path d="M5 10v2a7 7 0 0 0 14 0v-2"/><path d="M12 19v3"/></svg>
const IconChecklist = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>
const IconSend = () => <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
const IconReply = () => <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
const IconPlus = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
const IconGps = () => <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg>
const IconPhone = () => <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z"/></svg>
const IconSettings = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
const IconTrash = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
const IconCheck = () => <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"/></svg>

// Audio Visualizer Component
function AudioVisualizer({ stream }) {
  const canvasRef = useRef(null)
  const animationRef = useRef(null)
  
  useEffect(() => {
    if (!stream || !canvasRef.current) return
    const audioCtx = new (window.AudioContext || window.webkitAudioContext)()
    const analyser = audioCtx.createAnalyser()
    const source = audioCtx.createMediaStreamSource(stream)
    source.connect(analyser)
    analyser.fftSize = 64
    const bufferLength = analyser.frequencyBinCount
    const dataArray = new Uint8Array(bufferLength)
    const canvas = canvasRef.current
    const ctx = canvas.getContext('2d')

    const draw = () => {
      const width = canvas.width
      const height = canvas.height
      animationRef.current = requestAnimationFrame(draw)
      analyser.getByteFrequencyData(dataArray)
      ctx.clearRect(0, 0, width, height)
      
      const barWidth = (width / bufferLength) * 2.5
      let barHeight
      let x = 0
      
      for(let i = 0; i < bufferLength; i++) {
        barHeight = (dataArray[i] / 255) * height
        ctx.fillStyle = getComputedStyle(document.body).getPropertyValue('--primary').trim() || '#f97316'
        ctx.fillRect(x, height - barHeight, barWidth, barHeight)
        x += barWidth + 2
      }
    }
    draw()
    return () => { cancelAnimationFrame(animationRef.current); audioCtx.close() }
  }, [stream])

  return <canvas ref={canvasRef} width="150" height="40" className="audio-visualizer" />
}

// Custom Audio Player Component
// Uses native <audio> for playback (ignores iOS mute switch)
// Uses Web Audio API *only* to calculate duration to fix iOS Safari chunked duration bug
function AudioPlayer({ src }) {
  const [isPlaying, setIsPlaying] = useState(false)
  const [progress, setProgress] = useState(0)
  const [duration, setDuration] = useState(0)
  const [currentTime, setCurrentTime] = useState(0)
  const audioRef = useRef(null)

  // Fetch true duration using Web Audio API safely
  useEffect(() => {
    let isSubscribed = true
    const fetchDuration = async () => {
      try {
        const res = await fetch(src)
        const arrayBuffer = await res.arrayBuffer()
        const ctx = new (window.AudioContext || window.webkitAudioContext)()
        const decoded = await ctx.decodeAudioData(arrayBuffer)
        if (isSubscribed) setDuration(decoded.duration)
        if (ctx.state !== 'closed') ctx.close()
      } catch (e) {
         console.warn("Failed to get true duration", e)
      }
    }
    fetchDuration()
    return () => { isSubscribed = false }
  }, [src])

  useEffect(() => {
    const audio = audioRef.current
    if (!audio) return
    const updateProgress = () => {
       const ct = audio.currentTime
       setCurrentTime(ct)
       // Fallback calculating progress if duration hasn't loaded yet
       const dur = duration || audio.duration || 1
       setProgress((ct / dur) * 100)
    }
    const handleEnded = () => { setIsPlaying(false); setProgress(0); setCurrentTime(0) }
    
    audio.addEventListener('timeupdate', updateProgress)
    audio.addEventListener('ended', handleEnded)
    
    return () => {
      audio.removeEventListener('timeupdate', updateProgress)
      audio.removeEventListener('ended', handleEnded)
    }
  }, [duration])

  const togglePlay = () => {
    if (isPlaying) { audioRef.current.pause(); setIsPlaying(false) }
    else { audioRef.current.play(); setIsPlaying(true) }
  }

  const handleSeek = (e) => {
    const bounds = e.currentTarget.getBoundingClientRect()
    const x = e.clientX - bounds.left
    const perc = x / bounds.width
    // Use our fetched duration for seeking if audio.duration is buggy
    const dur = duration || audioRef.current.duration
    if (audioRef.current && dur && dur !== Infinity) {
      audioRef.current.currentTime = perc * dur
      setProgress(perc * 100)
      setCurrentTime(perc * dur)
    }
  }

  const formatTime = (secs) => {
    if (!secs || isNaN(secs) || secs === Infinity) return '0:00'
    const m = Math.floor(secs / 60)
    const s = Math.floor(secs % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  return (
    <div className="custom-audio-player">
      <audio ref={audioRef} src={src} preload="metadata" playsInline />
      <button className="audio-play-btn" onClick={togglePlay}>
        {isPlaying ? (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
        ) : (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
        )}
      </button>
      <div className="audio-timeline" onClick={handleSeek}>
        <div className="audio-progress" style={{ width: `${progress}%` }} />
      </div>
      <div className="audio-time">
        {formatTime(currentTime)} / {duration ? formatTime(duration) : '--:--'}
      </div>
    </div>
  )
}

// Linkify: detect URLs in text and render as clickable links
function Linkify({ text }) {
  if (!text) return null
  const urlRegex = /(https?:\/\/[^\s]+)/g
  const parts = text.split(urlRegex)
  return parts.map((part, i) => {
    if (urlRegex.test(part)) {
      urlRegex.lastIndex = 0 // reset regex state
      const isGoogleMaps = part.includes('google.com/maps') || part.includes('maps.google') || part.includes('goo.gl/maps') || part.includes('maps.app.goo.gl')
      if (isGoogleMaps) {
        return (
          <a key={i} href={part} target="_blank" rel="noopener noreferrer" className="link-embed maps-embed">
            <IconGps /> <span>Abrir no Google Maps</span>
          </a>
        )
      }
      // Generic link embed
      const domain = new URL(part).hostname.replace('www.', '')
      return (
        <a key={i} href={part} target="_blank" rel="noopener noreferrer" className="link-embed">
          <span className="link-domain">{domain}</span>
          <span className="link-url">{part.length > 50 ? part.slice(0, 50) + '...' : part}</span>
        </a>
      )
    }
    return <span key={i}>{part}</span>
  })
}

// Profile photo from localStorage
const getProfilePhoto = () => localStorage.getItem('charta-profile-photo')
const setProfilePhoto = (dataUrl) => localStorage.setItem('charta-profile-photo', dataUrl)

// Initials avatar (shows photo if matching current user or if profile exists in db)
// Tracks which URL failed so a new/changed URL auto-retries (like Discord)
function Avatar({ name, size = 36, profiles = {} }) {
  const [failedUrl, setFailedUrl] = useState(null)
  const currentName = getAuthorName()
  let photo = null
  const lowerName = name ? name.toLowerCase() : ''
  
  // 1. Try to get global profile photo for this user
  if (lowerName && profiles[lowerName] && profiles[lowerName].photo_url) {
    photo = profiles[lowerName].photo_url
  }
  // 2. Fallback to local profile photo if this avatar matches the current user
  else if (lowerName && currentName && lowerName === currentName.toLowerCase()) {
    photo = getProfilePhoto()
  }

  // Only skip loading if THIS specific URL already failed
  // If the URL changes (new upload, different user), it retries automatically
  if (photo && photo !== failedUrl) {
    return (
      <div className="avatar" style={{ width: size, height: size }}>
        <img
          src={photo}
          alt={name}
          style={{ width: '100%', height: '100%', borderRadius: '50%', objectFit: 'cover' }}
          onError={() => setFailedUrl(photo)}
          loading="lazy"
        />
      </div>
    )
  }

  const initials = (name || '?').split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2)
  const colors = ['#c17b4a', '#6b8e5a', '#5a7eb8', '#b85a7e', '#8e5ab8', '#b8a05a', '#5ab8a0']
  const idx = (name || '').split('').reduce((a, c) => a + c.charCodeAt(0), 0) % colors.length
  return (
    <div className="avatar" style={{ width: size, height: size, background: colors[idx], fontSize: size * 0.38 }}>
      {initials}
    </div>
  )
}

function App() {
  const [selectedProject, setSelectedProject] = useState(null)
  const [obraTab, setObraTab] = useState('fotos')
  const [showObraInfo, setShowObraInfo] = useState(false)
  
  // Voice recording state
  const [recordState, setRecordState] = useState('IDLE') // IDLE, RECORDING, PREVIEW
  const [recordStream, setRecordStream] = useState(null)
  const [recordBlob, setRecordBlob] = useState(null)
  const [recordDuration, setRecordDuration] = useState(0)
  const recordTimerRef = useRef(null)

  const [authenticated, setAuthenticated] = useState(false)
  const [apiKeyInput, setApiKeyInput] = useState('')
  const [authorInput, setAuthorInput] = useState(getAuthorName())
  const [showNamePrompt, setShowNamePrompt] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [profilePhoto, setProfilePhotoState] = useState(getProfilePhoto())
  const [isPhotoUploading, setIsPhotoUploading] = useState(false)
  const settingsPhotoRef = useRef(null)
  const [pushEnabled, setPushEnabled] = useState(false)
  const [isPushLoading, setIsPushLoading] = useState(true)

  // Check push subscription status on mount
  useEffect(() => {
    if ('serviceWorker' in navigator && 'PushManager' in window) {
      navigator.serviceWorker.ready.then(reg => {
        reg.pushManager.getSubscription().then(sub => {
          setPushEnabled(!!sub)
          setIsPushLoading(false)
        })
      })
    } else {
      setIsPushLoading(false)
    }
  }, [])

  const [projects, setProjects] = useState([])
  const [projectDocs, setProjectDocs] = useState([])
  const [forumPosts, setForumPosts] = useState([])
  const [userProfiles, setUserProfiles] = useState({}) // { name: { name, photo_url, updated_at } }
  const [loading, setLoading] = useState(false)
  const [uploading, setUploading] = useState(false)
  const [message, setMessage] = useState(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [newProjectName, setNewProjectName] = useState('')
  const [showNewProject, setShowNewProject] = useState(false)
  const [newProjectAddress, setNewProjectAddress] = useState('')
  const [newProjectPhone, setNewProjectPhone] = useState('')

  // Image detail view state: { post, document }
  const [imageDetail, setImageDetail] = useState(null)
  const [imageComments, setImageComments] = useState([])
  const [imageCommentText, setImageCommentText] = useState('')

  // Forum composer state
  const [showComposer, setShowComposer] = useState(false)
  const [composerType, setComposerType] = useState('TEXT') // TEXT | TASK_LIST
  const [composerText, setComposerText] = useState('')
  const [composerItems, setComposerItems] = useState([''])

  // Per-post reply state
  const [expandedReplies, setExpandedReplies] = useState({}) // postId -> { replies, loaded, replyText }
  const [postReplies, setPostReplies] = useState({}) // postId -> replies array
  const [replyTexts, setReplyTexts] = useState({}) // postId -> text

  // Voice
  const [isRecording, setIsRecording] = useState(false)
  const mediaRecorderRef = useRef(null)
  const projectNameInput = useState('')
  const audioChunksRef = useRef([])

  const [offlineQueueLength, setOfflineQueueLength] = useState(0)
  const [isSyncing, setIsSyncing] = useState(false)
  const [showArchived, setShowArchived] = useState(false)

  const fileInputRef = useRef(null)
  const forumEndRef = useRef(null)

  // ── Lifecycle ──
  useEffect(() => {
    checkAuth()
    checkQueueLength()
    loadProfiles()
    const handleOnline = () => syncOfflineQueue()
    window.addEventListener('online', handleOnline)
    return () => window.removeEventListener('online', handleOnline)
  }, [])

  // ── Auth ──
  const checkAuth = async () => {
    const key = getApiKey()
    if (!key) { setShowSettings(true); return }
    try {
      const res = await apiFetch(`${API_BASE}/projects`)
      if (res.status === 401) { setAuthenticated(false); setShowSettings(true) }
      else {
        setAuthenticated(true); loadProjects()
        if (!getAuthorName()) setShowNamePrompt(true)
      }
    } catch { setAuthenticated(false); setShowSettings(true) }
  }

  const saveSettings = () => {
    setApiKeyStore(apiKeyInput)
    setAuthorNameStore(authorInput)
    setShowSettings(false)
    checkAuth()
  }

  const saveName = () => {
    if (authorInput.trim()) { setAuthorNameStore(authorInput); setShowNamePrompt(false) }
  }

  const checkQueueLength = async () => { try { const q = await getUploadQueue(); setOfflineQueueLength(q.length) } catch {} }

  const syncOfflineQueue = async () => {
    if (isSyncing) return; setIsSyncing(true)
    try {
      const queue = await getUploadQueue()
      if (queue.length === 0) { setIsSyncing(false); return }
      showToast(`Sincronizando ${queue.length} pendentes...`)
      let ok = 0
      for (const item of queue) {
        try {
          const fd = new FormData()
          fd.append('file', item.file, item.originalName)
          if (item.projectId) fd.append('project_id', item.projectId)
          fd.append('author_name', getAuthorName() || 'Anónimo')
          const res = await apiFetch(`${API_BASE}/upload`, { method: 'POST', body: fd })
          if (res.ok) { await removeFromQueue(item.id); ok++ }
        } catch {}
      }
      await checkQueueLength()
      if (ok > 0) showToast(`${ok} sincronizados!`)
    } catch {}
    setIsSyncing(false)
  }

  const loadProjects = async () => {
    setLoading(true)
    try { const res = await apiFetch(`${API_BASE}/projects`); if (res.ok) setProjects(await res.json()) } catch {}
    setLoading(false)
  }

  const loadProfiles = async () => {
    try {
      const res = await apiFetch(`${API_BASE}/profiles`)
      if (res.ok) {
        const data = await res.json()
        const profileMap = {}
        data.forEach(p => profileMap[p.name.toLowerCase()] = p)
        setUserProfiles(profileMap)
        // Sync current user's profile photo from server to localStorage
        // This ensures profile photos set on other devices are available locally
        const currentName = getAuthorName()?.toLowerCase()
        if (currentName && profileMap[currentName]?.photo_url) {
          const serverPhoto = profileMap[currentName].photo_url
          const localPhoto = getProfilePhoto()
          if (serverPhoto && serverPhoto !== localPhoto) {
            setProfilePhoto(serverPhoto)
            setProfilePhotoState(serverPhoto)
          }
        }
      }
    } catch {}
  }

  const loadProjectDocs = async (pid) => {
    try { const res = await apiFetch(`${API_BASE}/projects/${pid}/documents`); if (res.ok) setProjectDocs(await res.json()) } catch {}
  }

  const shouldScrollRef = useRef(false)
  const scrollToBottom = () => {
    if (forumEndRef.current) {
      forumEndRef.current.scrollIntoView({ behavior: 'instant' })
      return true
    }
    return false
  }

  useEffect(() => {
    if (shouldScrollRef.current && forumPosts.length > 0) {
      // Use rAF + timeout to ensure DOM has painted before scrolling
      requestAnimationFrame(() => {
        setTimeout(() => {
          if (scrollToBottom()) {
            shouldScrollRef.current = false
          }
        }, 50)
      })
    }
  }, [forumPosts, obraTab])

  const loadForumPosts = async (pid) => {
    // Show cached data instantly
    const cached = await getCachedMessages(pid)
    if (cached) {
      setForumPosts(prev => prev.length === 0 ? cached : prev)
    }
    // Fetch fresh data
    try {
      const res = await apiFetch(`${API_BASE}/projects/${pid}/forum`)
      if (res.ok) {
        const data = await res.json()
        setForumPosts(data)
        cacheMessages(pid, data)
      }
    } catch {}
  }

  // Real-time polling for forum messages
  useEffect(() => {
    let interval
    if (selectedProject && (obraTab === 'forum' || selectedProject.name === 'Geral')) {
      interval = setInterval(() => {
        loadForumPosts(selectedProject.id)
        loadProfiles()
      }, 3000)
    }
    return () => clearInterval(interval)
  }, [selectedProject, obraTab])

  // ── Navigation ──
  const openProject = (project) => {
    setSelectedProject(project)
    setObraTab(project.name === 'Geral' ? 'forum' : 'fotos')
    setSearchQuery(''); setProjectDocs([]); setForumPosts([])
    setExpandedReplies({}); setPostReplies({}); setReplyTexts({})
    setShowComposer(false); cancelRecording()
    if (project.name !== 'Geral') loadProjectDocs(project.id)
    shouldScrollRef.current = true
    loadForumPosts(project.id)
    loadProfiles()
  }

  const goBackToList = () => {
    setSelectedProject(null);    setProjectDocs([]); setForumPosts([])
    setSearchQuery(''); setShowComposer(false); loadProjects()
    setShowArchived(false)
  }

  const switchTab = (tab) => {
    setObraTab(tab); setShowComposer(false)
    if (tab === 'forum' && selectedProject) {
      shouldScrollRef.current = true
      loadForumPosts(selectedProject.id)
    }
  }

  // ── File Upload ──
  const handleFileSelect = async (e) => {
    const files = e.target.files
    if (!files?.length || !selectedProject) return
    setUploading(true)
    let ok = 0, fail = 0, offline = 0
    for (const file of files) {
      try {
        const compressed = await compressImage(file)
        if (!navigator.onLine) { await saveUploadToQueue(compressed, file.name, null, selectedProject.id); offline++; continue }
        const fd = new FormData()
        fd.append('file', compressed, file.name)
        fd.append('project_id', selectedProject.id)
        fd.append('author_name', getAuthorName() || 'Anónimo')
        const res = await apiFetch(`${API_BASE}/upload`, { method: 'POST', body: fd })
        if (res.ok) ok++; else fail++
      } catch {
        try { const c = await compressImage(file); await saveUploadToQueue(c, file.name, null, selectedProject.id); offline++ } catch { fail++ }
      }
    }
    if (fileInputRef.current) fileInputRef.current.value = ''
    if (offline > 0) { showToast(`${offline} guardado(s) offline`); await checkQueueLength() }
    else if (fail === 0) showToast(`${ok} foto(s) enviada(s)!`)
    else showToast(`${ok} enviadas, ${fail} falharam`, 'error')
    setUploading(false); loadProjectDocs(selectedProject.id); loadForumPosts(selectedProject.id)
  }

  const triggerFileInput = (useCamera) => {
    if (!fileInputRef.current) return
    fileInputRef.current.setAttribute('accept', 'image/*')
    if (useCamera) fileInputRef.current.setAttribute('capture', 'environment')
    else fileInputRef.current.removeAttribute('capture')
    fileInputRef.current.click()
  }

  // ── Profile Photo Upload ──
  const handleProfilePhotoUpload = async (e) => {
    const file = e.target.files?.[0]
    if (!file) return
    setIsPhotoUploading(true)
    try {
      // 1. Compress image aggressively for profile photo
      const compressed = await compressImage(file, 400, 0.1)

      // 2. Upload to get the file URL (reusing the generic /upload endpoint)
      const fd = new FormData()
      fd.append('file', compressed, file.name)
      fd.append('author_name', getAuthorName() || 'Anónimo')
      
      const uploadRes = await apiFetch(`${API_BASE}/upload`, { method: 'POST', body: fd })
      if (!uploadRes.ok) throw new Error('Falha no upload da foto')
      
      const docData = await uploadRes.json()
      const photoUrl = docData.file_url

      // 3. Update global profile via PUT /api/profiles/:name/photo
      const profileName = getAuthorName() || 'Anónimo'
      const profileRes = await apiFetch(`${API_BASE}/profiles/${encodeURIComponent(profileName)}/photo`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ photo_url: photoUrl })
      })
      
      if (!profileRes.ok) throw new Error('Falha ao atualizar perfil')

      // 4. Update local state & storage
      setProfilePhoto(photoUrl)
      setProfilePhotoState(photoUrl)
      
      // 5. Refresh global profiles
      loadProfiles()
      
      showToast('Foto de perfil atualizada!')
    } catch (err) {
      showToast(err.message || 'Erro ao atualizar foto', 'error')
    } finally {
      setIsPhotoUploading(false)
      if (settingsPhotoRef.current) settingsPhotoRef.current.value = ''
    }
  }

  // ── Forum: Create Post ──
  const submitPost = async () => {
    if (!selectedProject) return
    if (composerType === 'TASK_LIST') {
      const items = composerItems.filter(t => t.trim())
      if (items.length === 0) return
      try {
        await apiFetch(`${API_BASE}/projects/${selectedProject.id}/forum`, {
          method: 'POST', headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ message_type: 'TASK_LIST', content: composerText || null, items, author_name: getAuthorName() || null })
        })
        resetComposer(); 
        shouldScrollRef.current = true;
        loadForumPosts(selectedProject.id)
      } catch { showToast('Erro ao criar post', 'error') }
    } else {
      if (!composerText.trim()) return
      try {
        await apiFetch(`${API_BASE}/projects/${selectedProject.id}/forum`, {
          method: 'POST', headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ message_type: 'TEXT', content: composerText, author_name: getAuthorName() || null })
        })
        resetComposer(); 
        shouldScrollRef.current = true;
        loadForumPosts(selectedProject.id)
      } catch { showToast('Erro ao criar post', 'error') }
    }
  }

  const resetComposer = () => {
    setShowComposer(false); setComposerText(''); setComposerItems(['']); setComposerType('TEXT'); cancelRecording()
  }

  // ── Forum: Replies ──
  const toggleReplies = async (postId) => {
    if (expandedReplies[postId]) {
      setExpandedReplies(prev => { const next = { ...prev }; delete next[postId]; return next })
      return
    }
    try {
      const res = await apiFetch(`${API_BASE}/forum/${postId}/replies`)
      if (res.ok) {
        const replies = await res.json()
        setPostReplies(prev => ({ ...prev, [postId]: replies }))
        setExpandedReplies(prev => ({ ...prev, [postId]: true }))
      }
    } catch {}
  }

  const submitReply = async (postId) => {
    const text = replyTexts[postId]?.trim()
    if (!text) return
    try {
      await apiFetch(`${API_BASE}/forum/${postId}/replies`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: text, author_name: getAuthorName() || null })
      })
      setReplyTexts(prev => ({ ...prev, [postId]: '' }))
      // Refresh replies
      const res = await apiFetch(`${API_BASE}/forum/${postId}/replies`)
      if (res.ok) {
        const replies = await res.json()
        setPostReplies(prev => ({ ...prev, [postId]: replies }))
      }
      loadForumPosts(selectedProject.id)
    } catch { showToast('Erro ao responder', 'error') }
  }

  // ── Forum: Voice ──
  const startRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      setRecordStream(stream)
      const supportedTypes = ['audio/webm', 'audio/mp4', 'audio/ogg']
      let options = undefined
      for (const t of supportedTypes) { if (MediaRecorder.isTypeSupported(t)) { options = { mimeType: t }; break } }
      mediaRecorderRef.current = options ? new MediaRecorder(stream, options) : new MediaRecorder(stream)
      audioChunksRef.current = []
      mediaRecorderRef.current.ondataavailable = (e) => { if (e.data?.size > 0) audioChunksRef.current.push(e.data) }
      
      setRecordState('RECORDING')
      setRecordDuration(0)
      recordTimerRef.current = setInterval(() => setRecordDuration(p => p + 1), 1000)
      mediaRecorderRef.current.start()
    } catch { showToast('Não foi possível aceder ao microfone', 'error') }
  }

  const stopRecordingAndPreview = () => {
    if (mediaRecorderRef.current && recordState === 'RECORDING') {
      mediaRecorderRef.current.onstop = () => {
        const mime = audioChunksRef.current[0]?.type || 'audio/webm'
        const blob = new Blob(audioChunksRef.current, { type: mime })
        setRecordBlob(blob)
        setRecordState('PREVIEW')
        if (recordStream) recordStream.getTracks().forEach(t => t.stop())
        setRecordStream(null)
      }
      try { if (mediaRecorderRef.current.state !== 'inactive') mediaRecorderRef.current.stop() } catch {}
      clearInterval(recordTimerRef.current)
    }
  }

  const cancelRecording = () => {
    if (mediaRecorderRef.current && mediaRecorderRef.current.state !== 'inactive') {
      mediaRecorderRef.current.stop()
    }
    if (recordStream) recordStream.getTracks().forEach(t => t.stop())
    setRecordStream(null)
    setRecordBlob(null)
    setRecordState('IDLE')
    clearInterval(recordTimerRef.current)
  }

  const sendVoiceMessage = async () => {
    if (!recordBlob) return
    const mime = recordBlob.type
    // Fallback to mp4 extension for better iOS handling of its own media containers
    const ext = mime.includes('mp4') || mime.includes('aac') ? 'mp4' : 'webm'
    const fd = new FormData()
    
    // Append text content BEFORE audio. Some multipart parsers consume streams sequentially!
    fd.append('author_name', getAuthorName() || 'Anónimo')
    if (composerText.trim()) fd.append('content', composerText.trim())
    
    fd.append('audio', recordBlob, `voice.${ext}`)
    
    setRecordState('IDLE'); setRecordBlob(null); setComposerText('')
    try { 
      await apiFetch(`${API_BASE}/projects/${selectedProject.id}/forum/voice`, { method: 'POST', body: fd })
      shouldScrollRef.current = true;
      loadForumPosts(selectedProject.id) 
    }
    catch { showToast('Erro ao enviar áudio', 'error') }
  }

  const toggleTaskItem = async (itemId) => {
    try {
      await apiFetch(`${API_BASE}/tasks/${itemId}/toggle`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ completed_by: getAuthorName() || null }) })
      loadForumPosts(selectedProject.id)
    } catch {}
  }

  const createProject = async () => {
    if (!newProjectName.trim()) return
    try {
      const body = { name: newProjectName }
      if (newProjectAddress.trim()) body.address = newProjectAddress.trim()
      if (newProjectPhone.trim()) body.client_phone = newProjectPhone.trim()
      const res = await apiFetch(`${API_BASE}/projects`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) })
      if (res.ok) { setNewProjectName(''); setNewProjectAddress(''); setNewProjectPhone(''); setShowNewProject(false); loadProjects(); showToast('Obra criada!') }
    } catch { showToast('Erro ao criar obra', 'error') }
  }

  const toggleProjectStatus = async (project) => {
    const newStatus = project.status === 'ACTIVE' ? 'ARCHIVED' : 'ACTIVE'
    try {
      await apiFetch(`${API_BASE}/projects/${project.id}/status`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ status: newStatus }) })
      setSelectedProject(prev => prev ? { ...prev, status: newStatus } : null); loadProjects()
    } catch {}
  }

  const showToast = (text, type = 'success') => { setMessage({ text, type }); setTimeout(() => setMessage(null), 3000) }

  // ── Image Detail ──
  const openImageDetail = async (post, doc) => {
    setImageDetail({ post, document: doc })
    setImageComments([])
    setImageCommentText('')
    if (post?.id) {
      try {
        const res = await apiFetch(`${API_BASE}/forum/${post.id}/replies`)
        if (res.ok) setImageComments(await res.json())
      } catch {}
    }
  }

  const closeImageDetail = () => {
    setImageDetail(null)
    setImageComments([])
    setImageCommentText('')
  }

  const submitImageComment = async () => {
    if (!imageCommentText.trim() || !imageDetail?.post?.id) return
    try {
      await apiFetch(`${API_BASE}/forum/${imageDetail.post.id}/replies`, {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: imageCommentText, author_name: getAuthorName() || null })
      })
      setImageCommentText('')
      const res = await apiFetch(`${API_BASE}/forum/${imageDetail.post.id}/replies`)
      if (res.ok) setImageComments(await res.json())
    } catch { showToast('Erro ao comentar', 'error') }
  }

  const formatTime = (dateStr) => {
    if (!dateStr) return ''
    const d = new Date(dateStr + 'Z')
    const now = new Date()
    const yesterday = new Date(); yesterday.setDate(yesterday.getDate() - 1)
    const time = d.toLocaleTimeString('pt-PT', { hour: '2-digit', minute: '2-digit' })
    if (d.toDateString() === now.toDateString()) return `Hoje, ${time}`
    if (d.toDateString() === yesterday.toDateString()) return `Ontem, ${time}`
    return `${d.toLocaleDateString('pt-PT', { day: '2-digit', month: '2-digit' })}, ${time}`
  }

  const isImage = (doc) => doc?.file_type === 'image' || doc?.original_name?.match(/\.(jpg|jpeg|png|gif|webp|heic)$/i)

  // ── RENDER ──

  // Settings (first run)
  if (!authenticated && showSettings) {
    return (
      <div className="app">
        <div className="settings-page">
          <h2>Configuração</h2>
          <label>Chave API</label>
          <input type="password" value={apiKeyInput} onChange={e => setApiKeyInput(e.target.value)} placeholder="Insira a chave API" />
          <label>O seu nome</label>
          <input type="text" value={authorInput} onChange={e => setAuthorInput(e.target.value)} placeholder="Ex: João" />
          <button className="btn-primary" onClick={saveSettings}>Guardar</button>
        </div>
      </div>
    )
  }



  // Push notification subscribe/unsubscribe
  const subscribeToPush = async () => {
    setIsPushLoading(true)
    try {
      if (!('serviceWorker' in navigator) || !('PushManager' in window)) {
        showToast('Push não suportado neste browser', 'error'); return
      }
      const permission = await Notification.requestPermission()
      if (permission !== 'granted') { showToast('Permissão negada', 'error'); return }

      // Get VAPID public key from server
      const keyRes = await apiFetch(`${API_BASE}/push/vapid-key`)
      if (!keyRes.ok) { showToast('Erro ao obter chave VAPID', 'error'); return }
      const { publicKey } = await keyRes.json()

      // Convert base64url to Uint8Array
      const padding = '='.repeat((4 - publicKey.length % 4) % 4)
      const raw = atob(publicKey.replace(/-/g, '+').replace(/_/g, '/') + padding)
      const key = new Uint8Array(raw.length)
      for (let i = 0; i < raw.length; i++) key[i] = raw.charCodeAt(i)

      const reg = await navigator.serviceWorker.ready
      const sub = await reg.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: key
      })

      const subJson = sub.toJSON()
      await apiFetch(`${API_BASE}/push/subscribe`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          endpoint: subJson.endpoint,
          p256dh: subJson.keys.p256dh,
          auth: subJson.keys.auth,
          author_name: getAuthorName() || null
        })
      })

      setPushEnabled(true)
      showToast('Notificações ativadas!')
    } catch (e) { showToast('Erro: ' + e.message, 'error') }
    finally { setIsPushLoading(false) }
  }

  const unsubscribeFromPush = async () => {
    setIsPushLoading(true)
    try {
      const reg = await navigator.serviceWorker.ready
      const sub = await reg.pushManager.getSubscription()
      if (sub) {
        await apiFetch(`${API_BASE}/push/unsubscribe`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ endpoint: sub.endpoint })
        })
        await sub.unsubscribe()
      }
      setPushEnabled(false)
      showToast('Notificações desativadas')
    } catch { showToast('Erro ao desativar', 'error') }
    finally { setIsPushLoading(false) }
  }

  // Settings page (authenticated)
  if (showSettings) {
    const userName = getAuthorName() || 'Anónimo'
    return (
      <div className="app">
        <div className="settings-page">
          <div className="settings-header">
            <h2>Definições</h2>
            <button className="settings-close" onClick={() => setShowSettings(false)}>Fechar</button>
          </div>

          <div className="settings-profile">
            <div className={`settings-avatar-wrap ${isPhotoUploading ? 'uploading' : ''}`} onClick={() => !isPhotoUploading && settingsPhotoRef.current?.click()}>
              {profilePhoto ? (
                <img src={profilePhoto} alt="Foto de perfil" className="settings-avatar-img" />
              ) : (
                <Avatar name={userName} size={80} profiles={userProfiles} />
              )}
              <div className="settings-avatar-overlay">
                {isPhotoUploading ? <div className="spinner-small" /> : <IconCamera />}
              </div>
            </div>
            <input ref={settingsPhotoRef} type="file" accept="image/*" onChange={handleProfilePhotoUpload} style={{display:'none'}} />
            <p className="settings-name">{userName}</p>
            <p className="settings-hint">{isPhotoUploading ? 'A guardar foto...' : 'Toque na foto para alterar'}</p>
          </div>

          {profilePhoto && (
            <button className="btn-cancel settings-remove-photo" onClick={async () => {
              // Clear server-side profile photo so it doesn't sync back
              const profileName = getAuthorName() || 'Anónimo'
              try {
                await apiFetch(`${API_BASE}/profiles/${encodeURIComponent(profileName)}/photo`, {
                  method: 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({ photo_url: '' })
                })
              } catch {}
              localStorage.removeItem('charta-profile-photo')
              setProfilePhotoState(null)
              loadProfiles()
              showToast('Foto removida')
            }}>Remover foto</button>
          )}

          <div className="settings-section">
            <label>Nome</label>
            <input type="text" value={authorInput} onChange={e => setAuthorInput(e.target.value)} placeholder="Ex: João" />
            <button className="btn-primary" onClick={() => { setAuthorNameStore(authorInput); showToast('Nome guardado!') }}>Guardar nome</button>
          </div>

          <div className="settings-section">
            <label>Notificações</label>
            <p className="settings-hint" style={{margin: '4px 0 10px'}}>Receba alertas quando houver novas mensagens nas obras.</p>
            {pushEnabled ? (
              <button className="btn-cancel" onClick={unsubscribeFromPush} style={{width:'100%'}} disabled={isPushLoading}>
                {isPushLoading ? 'A processar...' : 'Desativar notificações'}
              </button>
            ) : (
              <button className="btn-primary" onClick={subscribeToPush} style={{width:'100%'}} disabled={isPushLoading}>
                {isPushLoading ? 'A aguardar permissão...' : 'Ativar notificações'}
              </button>
            )}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="app">
      {message && <div className={`toast ${message.type}`}>{message.text}</div>}

      {/* Name prompt banner */}
      {showNamePrompt && (
        <div className="name-prompt-banner">
          <span>Insira o seu nome para o chat:</span>
          <div className="name-prompt-row">
            <input type="text" placeholder="Ex: João" value={authorInput} onChange={e => setAuthorInput(e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter') saveName() }} autoFocus />
            <button className="btn-primary" onClick={saveName} disabled={!authorInput.trim()}>OK</button>
            <button className="name-prompt-close" onClick={() => setShowNamePrompt(false)}>×</button>
          </div>
        </div>
      )}

      {/* ════════ OBRAS LIST ════════ */}
      {!selectedProject && (
        <>
          {offlineQueueLength > 0 && (
            <div style={{padding: '8px 16px'}}>
              <button className="badge-btn" onClick={syncOfflineQueue}>{offlineQueueLength} pendente{offlineQueueLength !== 1 ? 's' : ''}</button>
            </div>
          )}
          <div className="search-bar">
            <input type="text" placeholder="Procurar obra..." value={searchQuery} onChange={e => setSearchQuery(e.target.value)} />
            {searchQuery && <button className="search-clear" onClick={() => setSearchQuery('')}>×</button>}
          </div>
          <main className="main-content">
            <div className="obras-list">
              {/* Active Projects rendering */}
              {projects
                .filter(p => p.status === 'ACTIVE' || p.name === 'Geral')
                .filter(p => !searchQuery || p.name.toLowerCase().includes(searchQuery.toLowerCase()))
                .sort((a, b) => { if (a.name === 'Geral') return -1; if (b.name === 'Geral') return 1; return 0 })
                .map(p => (
                <div key={p.id} className={`obra-card ${p.name === 'Geral' ? 'geral' : ''}`} onClick={() => openProject(p)}>
                  <div className="obra-info">
                    <span className="obra-name">{p.name}</span>
                    <span className="obra-meta">
                      {p.name !== 'Geral' && <span className="status-pill active">Ativa</span>}
                      {p.document_count > 0 && <span>{p.document_count} fotos</span>}
                    </span>
                  </div>
                  <div className="obra-chevron">›</div>
                </div>
              ))}

              {/* Archived Projects Folder Toggle */}
              {projects.filter(p => p.status === 'ARCHIVED').length > 0 && !searchQuery && (
                <div className="arquivos-folder" onClick={() => setShowArchived(!showArchived)}>
                  <div className="obra-info">
                    <span className="obra-name" style={{ color: 'var(--text-2)' }}>Arquivos</span>
                    <span className="obra-meta">
                      <span>{projects.filter(p => p.status === 'ARCHIVED').length} obras</span>
                    </span>
                  </div>
                  <div className="obra-chevron" style={{ transform: showArchived ? 'rotate(90deg)' : 'rotate(0)' }}>›</div>
                </div>
              )}

              {/* Archived Projects Rendering */}
              {showArchived && !searchQuery && projects
                .filter(p => p.status === 'ARCHIVED')
                .map(p => (
                <div key={p.id} className="obra-card archived" onClick={() => openProject(p)}>
                  <div className="obra-info">
                    <span className="obra-name">{p.name}</span>
                    <span className="obra-meta">
                      <span className="status-pill archived">Arquivada</span>
                      {p.document_count > 0 && <span>{p.document_count} fotos</span>}
                    </span>
                  </div>
                  <div className="obra-chevron">›</div>
                </div>
              ))}

              {/* Search results fallback mapping (if searching, show all) */}
              {searchQuery && projects
                .filter(p => p.status === 'ARCHIVED')
                .filter(p => p.name.toLowerCase().includes(searchQuery.toLowerCase()))
                .map(p => (
                <div key={p.id} className="obra-card archived" onClick={() => openProject(p)}>
                  <div className="obra-info">
                    <span className="obra-name">{p.name}</span>
                    <span className="obra-meta">
                      <span className="status-pill archived">Arquivada</span>
                      {p.document_count > 0 && <span>{p.document_count} fotos</span>}
                    </span>
                  </div>
                  <div className="obra-chevron">›</div>
                </div>
              ))}

              {projects.length === 0 && !loading && (
                <div className="empty-state"><h3>Nenhuma Obra</h3><p>Crie uma nova obra para começar.</p></div>
              )}
            </div>
          </main>
          <button className="fab" onClick={() => setShowNewProject(true)}>+ Nova Obra</button>
          <button className="settings-fab" onClick={() => setShowSettings(true)}><IconSettings /></button>
        </>
      )}

      {/* ════════ OBRA DETAIL ════════ */}
      {selectedProject && (
        <>
          <div className="obra-header">
            <div className="obra-header-top">
              <h1 onClick={goBackToList}><span className="header-back">‹</span> {selectedProject.name}</h1>
              <div className="header-actions">
                {offlineQueueLength > 0 && (
                  <button className="badge-btn" onClick={syncOfflineQueue}>{offlineQueueLength} pendente{offlineQueueLength !== 1 ? 's' : ''}</button>
                )}
                {selectedProject.name !== 'Geral' && (
                  <button className="info-btn" onClick={() => setShowObraInfo(!showObraInfo)}>i</button>
                )}
              </div>
            </div>


            {selectedProject.name !== 'Geral' && (
              <div className="sub-tabs">
                <button className={obraTab === 'fotos' ? 'active' : ''} onClick={() => switchTab('fotos')}>Fotos</button>
                <button className={obraTab === 'forum' ? 'active' : ''} onClick={() => switchTab('forum')}>Fórum</button>
              </div>
            )}
          </div>

          <main className="main-content">
            <div className="obra-detail">

              {/* ──── FOTOS TAB ──── */}
              {obraTab === 'fotos' && selectedProject.name !== 'Geral' && (
                <div className="fotos-tab has-bottom-bar">
                  <input ref={fileInputRef} type="file" accept="image/*" capture="environment" multiple onChange={handleFileSelect} style={{display:'none'}} />
                  {uploading && <div className="upload-bar">A enviar...</div>}
                  {projectDocs.length === 0 ? (
                    <div className="empty-state"><h3>Sem Fotos</h3><p>Tire uma foto para adicioná-la a esta obra.</p></div>
                  ) : (
                    <div className="photo-grid">
                      {projectDocs.filter(d => isImage(d)).map(doc => (
                        <div key={doc.id} className="photo-card" onClick={() => openImageDetail(forumPosts.find(p => p.message_type === 'PHOTO' && p.document?.id === doc.id) || { author_name: doc.author_name, created_at: doc.created_at }, doc)}>
                          <img src={doc.file_url} alt={doc.original_name} loading="lazy" />
                          <span className="photo-name">{doc.original_name}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {/* ──── FÓRUM TAB ──── */}
              {(obraTab === 'forum' || selectedProject.name === 'Geral') && (
                <div className="forum-feed has-input-bar">
                  {/* GPS island */}
                  {selectedProject.address && selectedProject.name !== 'Geral' && (
                    <div className="gps-island">
                      <div className="gps-island-info">
                        <IconGps />
                        <span className="gps-address">{selectedProject.address}</span>
                      </div>
                      <div className="gps-island-actions">
                        {selectedProject.client_phone && (
                          <a href={`tel:${selectedProject.client_phone}`} className="gps-phone-btn"><IconPhone /> {selectedProject.client_phone}</a>
                        )}
                        <a href={`https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(selectedProject.address)}`}
                          target="_blank" rel="noopener noreferrer" className="gps-nav-btn">
                          Abrir no GPS
                        </a>
                      </div>
                    </div>
                  )}

                  {/* Posts feed */}
                  {forumPosts.length === 0 && (
                    <div className="empty-state small"><p>Sem publicações nesta obra.</p></div>
                  )}

                  {forumPosts.map(post => (
                    <div key={post.id} className="post-card">
                      {/* Post header */}
                      <div className="post-header">
                        <Avatar name={post.author_name} size={32} profiles={userProfiles} />
                        <div className="post-meta">
                          <strong>{post.author_name || 'Anónimo'}</strong>
                          <span>{formatTime(post.created_at)}</span>
                        </div>
                      </div>

                      {/* Post content */}
                      {post.message_type === 'TEXT' && (
                        <div className="post-body"><Linkify text={post.content} /></div>
                      )}

                      {post.message_type === 'PHOTO' && post.document && (
                        <div className="post-body">
                          <div className="post-photo" onClick={() => openImageDetail(post, post.document)}>
                            <img src={post.document.file_url} alt={post.document.original_name} loading="lazy" />
                          </div>
                        </div>
                      )}

                      {post.message_type === 'VOICE' && post.audio_url && (
                        <div className="post-body">
                          {post.content && <p className="voice-message-text"><Linkify text={post.content} /></p>}
                          <AudioPlayer src={post.audio_url} />
                        </div>
                      )}

                      {post.message_type === 'TASK_LIST' && (
                        <div className="post-body">
                          {post.content && <div className="tasklist-title">{post.content}</div>}
                          <div className="tasklist">
                            {post.items?.map(item => (
                              <label key={item.id} className={`task-item ${item.completed ? 'done' : ''}`}>
                                <input type="checkbox" checked={item.completed} onChange={() => toggleTaskItem(item.id)} />
                                <span className="task-text">{item.text}</span>
                                {item.completed_by && <span className="task-by">{item.completed_by}</span>}
                              </label>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Replies section */}
                      <div className="post-footer">
                        <button className="reply-toggle" onClick={() => toggleReplies(post.id)}>
                          <IconReply />
                          <span>{post.reply_count > 0 ? `${post.reply_count} resposta${post.reply_count !== 1 ? 's' : ''}` : 'Responder'}</span>
                        </button>
                      </div>

                      {expandedReplies[post.id] && (
                        <div className="replies-section">
                          {(postReplies[post.id] || []).map(reply => (
                            <div key={reply.id} className="reply-item">
                              <Avatar name={reply.author_name} size={28} profiles={userProfiles} />
                              <div className="reply-content">
                                <div className="reply-meta">
                                  <strong>{reply.author_name || 'Anónimo'}</strong>
                                  <span>{formatTime(reply.created_at)}</span>
                                </div>
                                <p>{reply.content}</p>
                              </div>
                            </div>
                          ))}

                          <div className="reply-input-row">
                            <input type="text" value={replyTexts[post.id] || ''} onChange={e => setReplyTexts(prev => ({ ...prev, [post.id]: e.target.value }))}
                              placeholder="Escrever resposta..." onKeyDown={e => { if (e.key === 'Enter') submitReply(post.id) }} />
                            <button className="btn-send" onClick={() => submitReply(post.id)} disabled={!(replyTexts[post.id] || '').trim()}>
                              Enviar
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                  <div ref={forumEndRef} />
                </div>
              )}
            </div>
          </main>

          {/* Bottom action bar (Fotos) */}
          {obraTab === 'fotos' && selectedProject.name !== 'Geral' && (
            <div className="bottom-action-bar">
              <button className="primary-action" onClick={() => triggerFileInput(true)} title="Tirar foto"><IconCamera /></button>
              <button onClick={() => triggerFileInput(false)} title="Escolher ficheiro"><IconUpload /></button>
            </div>
          )}

          {/* Forum input bar */}
          {(obraTab === 'forum' || selectedProject.name === 'Geral') && (
            <div className="forum-input-bar">
              {composerType === 'TASK_LIST' && (
                <div className="task-panel">
                  <div className="task-panel-header">
                    <span>Lista de tarefas</span>
                    <button onClick={() => { setComposerType('TEXT'); setComposerItems(['']) }}>×</button>
                  </div>
                  <input type="text" value={composerText} onChange={e => setComposerText(e.target.value)} placeholder="Título (opcional)" className="task-panel-title" />
                  {composerItems.map((item, i) => (
                    <div key={i} className="task-panel-row">
                      <input type="text" value={item} onChange={e => { const c = [...composerItems]; c[i] = e.target.value; setComposerItems(c) }}
                        placeholder={`Item ${i + 1}`} onKeyDown={e => { if (e.key === 'Enter') setComposerItems([...composerItems, '']) }} />
                      {composerItems.length > 1 && <button className="remove-item" onClick={() => setComposerItems(composerItems.filter((_, j) => j !== i))}>×</button>}
                    </div>
                  ))}
                  <button className="add-item-btn" onClick={() => setComposerItems([...composerItems, ''])}>+ Adicionar item</button>
                  <button className="btn-primary task-panel-send" onClick={submitPost}>Publicar Lista</button>
                </div>
              )}
              <div className="forum-input-row">
                {recordState === 'IDLE' ? (
                  <>
                    <button className="input-action" title="Lista de tarefas" onClick={() => setComposerType(composerType === 'TASK_LIST' ? 'TEXT' : 'TASK_LIST')}>
                      <IconChecklist />
                    </button>
                    <button className="input-action" onClick={startRecording} title="Mensagem de voz">
                      <IconMic />
                    </button>
                    <input type="text" value={composerText} onChange={e => setComposerText(e.target.value)}
                      placeholder="Mensagem..." onKeyDown={e => { if (e.key === 'Enter' && composerType === 'TEXT') submitPost() }} />
                    <button className="send-btn" onClick={submitPost} disabled={composerType === 'TEXT' && !composerText.trim()}><IconSend /></button>
                  </>
                ) : recordState === 'RECORDING' ? (
                  <div className="voice-recording-bar">
                    <button className="voice-cancel" onClick={cancelRecording}><IconTrash /></button>
                    <div className="voice-visualizer">
                      <div className="voice-time">
                        {Math.floor(recordDuration / 60)}:{(recordDuration % 60).toString().padStart(2, '0')}
                      </div>
                      <AudioVisualizer stream={recordStream} />
                    </div>
                    <button className="voice-done" onClick={stopRecordingAndPreview}><IconCheck /></button>
                  </div>
                ) : (
                  <div className="voice-preview-bar">
                    <button className="voice-cancel" onClick={cancelRecording}><IconTrash /></button>
                    <div className="voice-preview-content">
                      <div className="voice-audio-preview">Áudio - {(recordBlob.size / 1024).toFixed(0)} KB  |  {Math.floor(recordDuration / 60)}:{(recordDuration % 60).toString().padStart(2, '0')}</div>
                      <input type="text" value={composerText} onChange={e => setComposerText(e.target.value)}
                        placeholder="Adicionar texto (opcional)..." onKeyDown={e => { if (e.key === 'Enter') sendVoiceMessage() }} />
                    </div>
                    <button className="send-btn" onClick={sendVoiceMessage}><IconSend /></button>
                  </div>
                )}
              </div>
            </div>
          )}
        </>
      )}

      {/* Modals */}
      {showNewProject && (
        <div className="modal-overlay" onClick={() => setShowNewProject(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <h3>Nova Obra</h3>
            <input type="text" placeholder="Nome da obra" value={newProjectName} onChange={e => setNewProjectName(e.target.value)} autoFocus />
            <input type="text" placeholder="Morada (opcional)" value={newProjectAddress} onChange={e => setNewProjectAddress(e.target.value)} />
            <input type="tel" placeholder="Telefone do cliente (opcional)" value={newProjectPhone} onChange={e => setNewProjectPhone(e.target.value)} onKeyDown={e => e.key === 'Enter' && createProject()} />
            <div className="modal-actions">
              <button className="btn-cancel" onClick={() => setShowNewProject(false)}>Cancelar</button>
              <button className="btn-primary" onClick={createProject}>Criar</button>
            </div>
          </div>
        </div>
      )}

      {/* Obra Info Modal */}
      {showObraInfo && selectedProject && selectedProject.name !== 'Geral' && (
        <div className="modal-overlay" onClick={() => setShowObraInfo(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <button className="modal-close-text" onClick={() => setShowObraInfo(false)}>Fechar</button>
            <h3 style={{ textAlign: 'center', marginTop: '4px' }}>{selectedProject.name}</h3>
            {selectedProject.address && (
              <div className="info-row">
                <IconGps />
                <div className="info-row-content">
                  <span className="info-label">Morada</span>
                  <span>{selectedProject.address}</span>
                  <a href={`https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(selectedProject.address)}`}
                    target="_blank" rel="noopener noreferrer" className="info-link">Abrir no GPS</a>
                </div>
              </div>
            )}
            {selectedProject.client_phone && (
              <div className="info-row">
                <IconPhone />
                <div className="info-row-content">
                  <span className="info-label">Telefone do cliente</span>
                  <a href={`tel:${selectedProject.client_phone}`} className="info-link">{selectedProject.client_phone}</a>
                </div>
              </div>
            )}
            {!selectedProject.address && !selectedProject.client_phone && (
              <p className="info-empty">Sem morada ou telefone. Adicione ao criar a obra.</p>
            )}
            
            <button className="info-archive-btn" onClick={() => { toggleProjectStatus(selectedProject); setShowObraInfo(false) }} style={{ marginTop: '20px' }}>
              {selectedProject.status === 'ACTIVE' ? 'Arquivar Obra' : 'Reativar Obra'}
            </button>
          </div>
        </div>
      )}

      {/* Image Detail View */}
      {imageDetail && (
        <div className="image-detail-page">
          <div className="image-detail-header">
            <div className="image-detail-author">
              <Avatar name={imageDetail.post?.author_name} profiles={userProfiles} />
              <div>
                <strong>{imageDetail.post?.author_name || 'Anónimo'}</strong>
                <span className="post-time">{formatTime(imageDetail.post?.created_at)}</span>
              </div>
            </div>
            <button className="image-detail-close" onClick={closeImageDetail}>Fechar</button>
          </div>

          <div className="image-detail-body">
            <img src={imageDetail.document?.file_url} alt={imageDetail.document?.original_name} />
          </div>

          {imageDetail.post?.id && (
            <div className="image-detail-comments">
              <h4>Comentários ({imageComments.length})</h4>
              {imageComments.map(c => (
                <div key={c.id} className="image-comment">
                  <div className="image-comment-header">
                    <strong>{c.author_name || 'Anónimo'}</strong>
                  </div>
                  <p>{c.content}</p>
                  <span className="post-time">{formatTime(c.created_at)}</span>
                </div>
              ))}
              <div className="image-comment-input">
                <input type="text" value={imageCommentText} onChange={e => setImageCommentText(e.target.value)}
                  placeholder="Escrever comentário..." onKeyDown={e => { if (e.key === 'Enter') submitImageComment() }} />
                <button className="btn-send" onClick={submitImageComment} disabled={!imageCommentText.trim()}>Enviar</button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default App
