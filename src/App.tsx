import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { homeDir, join } from '@tauri-apps/api/path'
import { open } from '@tauri-apps/plugin-shell'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { Save, X, Music, Youtube, Globe, Play, Layers } from 'lucide-react'
import TitleBar from './components/TitleBar'
import './App.css'

interface DownloadProgress {
  percent: number
  speed: string
  eta: string
}

interface ArchiveItem {
  id: string
  title: string
  url: string
  platform: string
  date: string
  path: string
  format: 'mp3' | 'mp4'
}

function App() {
  const [url, setUrl] = useState('')
  const [isDownloading, setIsDownloading] = useState(false)
  const [progress, setProgress] = useState<DownloadProgress | null>(null)
  const [status, setStatus] = useState<'idle' | 'downloading' | 'success' | 'error'>('idle')
  const [platform, setPlatform] = useState<string | null>(null)
  const [archiveOpen, setArchiveOpen] = useState(false)
  const [archive, setArchive] = useState<ArchiveItem[]>([])
  const [downloadFormat, setDownloadFormat] = useState<'mp3' | 'mp4'>('mp4')
  const [archiveTab, setArchiveTab] = useState<'all' | 'video' | 'audio'>('all')
  const inputRef = useRef<HTMLInputElement>(null)
  const archivePanelRef = useRef<HTMLDivElement>(null)
  const downloadInfoRef = useRef<{url: string, platform: string, format: 'mp3' | 'mp4'} | null>(null)

  useEffect(() => {
    // Listen for download progress
    const progressUnsubscribe = listen<DownloadProgress>('download-progress', (event) => {
      console.log('Progress event:', event.payload)
      setProgress(event.payload)
      setStatus('downloading')
    })

    // Listen for download started
    const startedUnsubscribe = listen<string>('download-started', (event) => {
      console.log('Download started:', event.payload)
      setStatus('downloading')
    })

    // Listen for download status messages (from stderr)
    const statusUnsubscribe = listen<string>('download-status', (event) => {
      console.log('Status message:', event.payload)
    })

    // Listen for download completion
    const completeUnsubscribe = listen<{success: boolean, path?: string, error?: string}>('download-complete', (event) => {
      console.log('Download complete:', event.payload)

      if (event.payload.success && event.payload.path && downloadInfoRef.current) {
        // Add to archive
        const newItem: ArchiveItem = {
          id: Date.now().toString(),
          title: downloadInfoRef.current.url.split('/').pop() || 'Download',
          url: downloadInfoRef.current.url,
          platform: downloadInfoRef.current.platform,
          date: new Date().toLocaleDateString(),
          path: event.payload.path,
          format: downloadInfoRef.current.format
        }

        const newArchive = [newItem, ...archive]
        setArchive(newArchive)
        localStorage.setItem('ripvid-archive', JSON.stringify(newArchive))
        console.log('Added to archive:', newItem)
      }

      setStatus(event.payload.success ? 'success' : 'error')
      setIsDownloading(false)
      downloadInfoRef.current = null
    })

    return () => {
      progressUnsubscribe.then(fn => fn())
      startedUnsubscribe.then(fn => fn())
      statusUnsubscribe.then(fn => fn())
      completeUnsubscribe.then(fn => fn())
    }
  }, [archive])

  useEffect(() => {
    // Show window once app is ready
    const showWindow = async () => {
      const appWindow = getCurrentWebviewWindow()
      await appWindow.show()
    }
    showWindow()

    // Load archive from localStorage
    const saved = localStorage.getItem('ripvid-archive')
    if (saved) {
      setArchive(JSON.parse(saved))
    }
    // Load format preference
    const savedFormat = localStorage.getItem('ripvid-format')
    if (savedFormat === 'mp3' || savedFormat === 'mp4') {
      setDownloadFormat(savedFormat)
    }
  }, [])

  useEffect(() => {
    // Click outside to close archive
    const handleClickOutside = (event: MouseEvent) => {
      if (archiveOpen &&
          archivePanelRef.current &&
          !archivePanelRef.current.contains(event.target as Node) &&
          !(event.target as Element).closest('.archive-toggle')) {
        setArchiveOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [archiveOpen])

  useEffect(() => {
    if (status === 'success' || status === 'error') {
      const timer = setTimeout(() => {
        setStatus('idle')
        setProgress(null)
        setUrl('')
        if (inputRef.current) {
          inputRef.current.focus()
        }
      }, 3000)
      return () => clearTimeout(timer)
    }
  }, [status])

  const detectPlatform = async (videoUrl: string) => {
    try {
      const detected = await invoke<string>('detect_platform', { url: videoUrl })
      setPlatform(detected)
      return detected
    } catch (error) {
      console.error('Failed to detect platform:', error)
      setPlatform(null)
      return null
    }
  }

  const handleUrlChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const newUrl = e.target.value
    setUrl(newUrl)

    if (newUrl.trim()) {
      await detectPlatform(newUrl)
    } else {
      setPlatform(null)
    }
  }

  const getDownloadPath = async () => {
    const home = await homeDir()
    const formatFolder = downloadFormat.toUpperCase()
    const ripvidDir = await join(home, 'Videos', 'ripVID', formatFolder)

    // Create directory if it doesn't exist
    await invoke('create_directory', { path: ripvidDir })

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5)
    const filename = `${platform}_${timestamp}.${downloadFormat}`

    return await join(ripvidDir, filename)
  }

  const handleDownload = async () => {
    if (!url.trim() || !platform || isDownloading) return

    console.log('Starting download:', { url, platform, format: downloadFormat })

    setIsDownloading(true)
    setStatus('downloading')
    setProgress(null)

    // Store download info for later use in completion handler
    downloadInfoRef.current = {
      url: url.trim(),
      platform: platform,
      format: downloadFormat
    }

    try {
      const savePath = await getDownloadPath()
      console.log('Save path:', savePath)

      // Use different command based on format
      if (downloadFormat === 'mp3') {
        console.log('Downloading as MP3...')
        const result = await invoke('download_audio', {
          url: url.trim(),
          outputPath: savePath
        })
        console.log('Audio download started:', result)
      } else {
        console.log('Downloading as MP4...')
        const result = await invoke('download_video', {
          url: url.trim(),
          outputPath: savePath,
          quality: 'best'
        })
        console.log('Video download started:', result)
      }

      // The actual completion and archive addition will be handled by the download-complete event
    } catch (error) {
      console.error('Failed to start download:', error)
      setStatus('error')
      setIsDownloading(false)
      downloadInfoRef.current = null
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleDownload()
    } else if (e.key === 'Escape') {
      setUrl('')
      setPlatform(null)
      setStatus('idle')
    } else if (e.key === 'Tab') {
      e.preventDefault()
      setArchiveOpen(!archiveOpen)
    }
  }

  const openFolder = async (path: string) => {
    try {
      // On Windows, open explorer and select the file
      if (navigator.platform.includes('Win')) {
        // Use Windows Explorer with /select flag to highlight the file
        await invoke('open_file_location', { path: path })
      } else {
        // For other platforms, just open the containing folder
        const folder = path.substring(0, path.lastIndexOf('/'))
        await open(folder)
      }
    } catch (error) {
      console.error('Failed to open folder:', error)
      // Fallback: try to open just the folder
      try {
        const folder = path.substring(0, Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\')))
        await open(folder)
      } catch (fallbackError) {
        console.error('Fallback also failed:', fallbackError)
      }
    }
  }

  const deleteFromArchive = async (id: string) => {
    const item = archive.find(item => item.id === id)
    if (!item) return

    try {
      // Recycle the actual file
      await invoke('recycle_file', { path: item.path })

      // Remove from archive
      const newArchive = archive.filter(item => item.id !== id)
      setArchive(newArchive)
      localStorage.setItem('ripvid-archive', JSON.stringify(newArchive))
    } catch (error) {
      console.error('Failed to recycle file:', error)
      // Still remove from archive even if file recycling fails
      const newArchive = archive.filter(item => item.id !== id)
      setArchive(newArchive)
      localStorage.setItem('ripvid-archive', JSON.stringify(newArchive))
    }
  }

  const toggleFormat = () => {
    const newFormat = downloadFormat === 'mp4' ? 'mp3' : 'mp4'
    setDownloadFormat(newFormat)
    localStorage.setItem('ripvid-format', newFormat)
  }

  const getFilteredArchive = () => {
    if (archiveTab === 'all') return archive
    if (archiveTab === 'video') return archive.filter(item => item.format === 'mp4')
    if (archiveTab === 'audio') return archive.filter(item => item.format === 'mp3')
    return archive
  }

  const getPlatformIcon = (size = 14) => {
    if (platform === 'youtube') return <Youtube size={size} />
    if (platform === 'x') return <Globe size={size} />
    return null
  }

  const getStatusContent = () => {
    if (isDownloading && progress) {
      return (
        <div className="progress-text">
          <span className="progress-platform">{getPlatformIcon()}</span>
          <span className="progress-percent">{Math.round(progress.percent)}%</span>
          <span className="progress-separator">•</span>
          <span className="progress-speed">{progress.speed}</span>
          <span className="progress-separator">•</span>
          <span className="progress-eta">ETA {progress.eta}</span>
        </div>
      )
    }

    if (status === 'success') {
      return <div className="success-text">Download complete</div>
    }

    if (status === 'error') {
      return <div className="error-text">Download failed</div>
    }

    return null
  }

  return (
    <>
      <TitleBar />
      <div className="app">
        <div className="logo">
          <span className="logo-text">rip</span>
          <span className="logo-v">V</span>
          <span className="logo-text">ID</span>
        </div>

        <button
          className={`format-toggle ${downloadFormat}`}
        onClick={toggleFormat}
        aria-label={`Switch to ${downloadFormat === 'mp4' ? 'MP3' : 'MP4'}`}
      >
        <div className="format-toggle-inner">
          <div className="format-option mp4">
            <Play size={14} />
          </div>
          <div className="format-option mp3">
            <Music size={14} />
          </div>
        </div>
      </button>

      <div className="input-container">
        <div className={`input-wrapper ${isDownloading ? 'downloading' : ''}`}>
          <input
            ref={inputRef}
            type="url"
            placeholder="Paste URL here..."
            value={url}
            onChange={handleUrlChange}
            onKeyDown={handleKeyPress}
            className="main-input"
            disabled={isDownloading}
            autoFocus
          />
        </div>
        <div className={`status-info ${status !== 'idle' ? 'active' : ''}`}>
          {getStatusContent()}
        </div>
      </div>

      {!archiveOpen && (
        <button
          className="archive-toggle"
          onClick={() => setArchiveOpen(true)}
          aria-label="Open archive"
        >
          <Save size={18} />
        </button>
      )}

      <div ref={archivePanelRef} className={`archive-panel ${archiveOpen ? 'open' : ''}`}>
        <div className="archive-header">
          <div className="archive-tabs">
            <button
              className={`archive-tab ${archiveTab === 'all' ? 'active' : ''}`}
              onClick={() => setArchiveTab('all')}
              title={`All (${archive.length})`}
            >
              <Layers size={20} />
            </button>
            <span className="tab-divider">|</span>
            <button
              className={`archive-tab ${archiveTab === 'video' ? 'active' : ''}`}
              onClick={() => setArchiveTab('video')}
              title={`Videos (${archive.filter(i => i.format === 'mp4').length})`}
            >
              <Play size={20} />
            </button>
            <span className="tab-divider">|</span>
            <button
              className={`archive-tab ${archiveTab === 'audio' ? 'active' : ''}`}
              onClick={() => setArchiveTab('audio')}
              title={`Audio (${archive.filter(i => i.format === 'mp3').length})`}
            >
              <Music size={20} />
            </button>
          </div>
        </div>

        {getFilteredArchive().length > 0 ? (
          <div className="archive-list">
            {getFilteredArchive().map(item => (
              <div
                key={item.id}
                className="archive-item"
              >
                <div className="archive-item-content" onClick={() => openFolder(item.path)}>
                  <span className="archive-item-name">{item.title}</span>
                  <span className={`archive-item-type ${item.format === 'mp3' ? 'audio' : 'video'}`}>{item.format?.toUpperCase()}</span>
                  <span className="archive-item-date">{item.date}</span>
                </div>
                <button
                  className="archive-item-delete"
                  onClick={async (e) => {
                    e.stopPropagation()
                    await deleteFromArchive(item.id)
                  }}
                  aria-label="Delete from archive"
                >
                  <X size={13} />
                </button>
              </div>
            ))}
          </div>
        ) : (
          <div className="archive-empty">
            No downloads yet
          </div>
        )}
        </div>
      </div>
    </>
  )
}

export default App