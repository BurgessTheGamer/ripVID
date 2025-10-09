import { useState } from 'react'
import { X, AlertTriangle, ExternalLink } from 'lucide-react'

interface TermsAcceptanceProps {
  onAccept: () => void
  onDecline: () => void
}

export function TermsAcceptance({ onAccept, onDecline }: TermsAcceptanceProps) {
  const [scrolledToBottom, setScrolledToBottom] = useState(false)

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const element = e.currentTarget
    const isAtBottom = Math.abs(element.scrollHeight - element.scrollTop - element.clientHeight) < 10
    setScrolledToBottom(isAtBottom)
  }

  const openGitHub = () => {
    window.open('https://github.com/BurgessTheGamer/ripVID/blob/main/TERMS.md', '_blank')
  }

  return (
    <div className="terms-overlay">
      <div className="terms-modal">
        <div className="terms-header">
          <AlertTriangle size={24} className="terms-icon" />
          <h2>Terms of Service & License Agreement</h2>
          <button className="terms-close" onClick={onDecline}>
            <X size={20} />
          </button>
        </div>

        <div className="terms-content" onScroll={handleScroll}>
          <div className="terms-important">
            <strong>IMPORTANT:</strong> By using ripVID, you agree to be bound by these terms.
          </div>

          <h3>1. User Responsibilities</h3>
          <p>You are solely responsible for:</p>
          <ul>
            <li>Complying with all applicable laws in your jurisdiction</li>
            <li>Respecting copyright and intellectual property rights</li>
            <li>Obtaining necessary permissions before downloading content</li>
            <li>Using the software only for lawful purposes</li>
            <li>Complying with the terms of service of third-party platforms</li>
          </ul>

          <h3>2. Prohibited Uses</h3>
          <p>You may NOT use this software to:</p>
          <ul>
            <li>Download copyrighted content without permission</li>
            <li>Violate any laws or regulations</li>
            <li>Infringe upon the rights of others</li>
            <li>Circumvent DRM or other protective measures</li>
          </ul>

          <h3>3. Disclaimer of Warranties</h3>
          <p>
            THIS SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
            INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
            PARTICULAR PURPOSE AND NONINFRINGEMENT.
          </p>

          <h3>4. Limitation of Liability</h3>
          <p>
            IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
            DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
            ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE.
          </p>

          <h3>5. Privacy</h3>
          <p>
            ripVID does not collect or transmit personal data. All downloads are processed
            locally on your device.
          </p>

          <h3>6. Copyright Notice</h3>
          <p>
            Copyright (c) 2024 ripVID. All rights reserved.<br />
            Licensed under the Apache License, Version 2.0
          </p>

          <div className="terms-scroll-hint">
            {!scrolledToBottom && "Please scroll to read all terms"}
          </div>
        </div>

        <div className="terms-footer">
          <button
            className="terms-github"
            onClick={openGitHub}
          >
            <ExternalLink size={16} />
            View Full Terms on GitHub
          </button>

          <div className="terms-actions">
            <button
              className="terms-decline"
              onClick={onDecline}
            >
              Decline
            </button>
            <button
              className={`terms-accept ${!scrolledToBottom ? 'disabled' : ''}`}
              onClick={onAccept}
              disabled={!scrolledToBottom}
              title={!scrolledToBottom ? "Please read all terms first" : "Accept terms and continue"}
            >
              I Accept
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}