import { useState, useEffect } from 'react'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { Minus, Square, X } from 'lucide-react'
import './TitleBar.css'

function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false)
  const appWindow = getCurrentWebviewWindow()

  useEffect(() => {
    const checkMaximized = async () => {
      const maximized = await appWindow.isMaximized()
      setIsMaximized(maximized)
    }
    checkMaximized()
  }, [])

  const handleMinimize = () => {
    appWindow.minimize()
  }

  const handleMaximize = async () => {
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      await appWindow.unmaximize()
      setIsMaximized(false)
    } else {
      await appWindow.maximize()
      setIsMaximized(true)
    }
  }

  const handleClose = () => {
    appWindow.close()
  }

  return (
    <div className="title-bar" data-tauri-drag-region>
      <div className="window-controls">
        <button
          className="control-button minimize"
          onClick={handleMinimize}
          aria-label="Minimize"
        >
          <Minus size={14} />
        </button>
        <button
          className="control-button maximize"
          onClick={handleMaximize}
          aria-label={isMaximized ? "Restore" : "Maximize"}
        >
          <Square size={11} />
        </button>
        <button
          className="control-button close"
          onClick={handleClose}
          aria-label="Close"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  )
}

export default TitleBar