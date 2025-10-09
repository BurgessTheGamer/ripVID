/**
 * Shared TypeScript interfaces and types for ripVID Desktop App
 */

export interface DownloadProgress {
  percent: number
  speed: string
  eta: string
}

export interface ArchiveItem {
  id: string
  title: string
  url: string
  platform: string
  date: string
  path: string
  format: 'mp3' | 'mp4'
}

export type DownloadStatus = 'idle' | 'downloading' | 'success' | 'error'

export type VideoQuality = 'best' | '2160p' | '1440p' | '1080p' | '720p' | '480p' | '360p'

export type BrowserType = 'firefox' | 'chrome' | 'edge' | 'brave' | 'safari' | 'opera'

export interface QualityOption {
  value: VideoQuality
  label: string
  description: string
}

export interface BrowserOption {
  value: BrowserType
  label: string
  available: boolean
}

export interface DownloadSettings {
  quality: VideoQuality
  useCookies: boolean
  browser: BrowserType | null
}
