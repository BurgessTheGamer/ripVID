import { Component, ErrorInfo, ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import "./ErrorBoundary.css";

interface Props {
    children: ReactNode;
    fallback?: ReactNode;
    onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
    hasError: boolean;
    error: Error | null;
    errorInfo: ErrorInfo | null;
}

/**
 * ErrorBoundary component for desktop app that catches JavaScript errors anywhere in the child component tree,
 * logs those errors, and displays a fallback UI instead of crashing the whole application.
 *
 * Features:
 * - Displays user-friendly error messages
 * - Provides recovery options (reload, reset)
 * - Logs errors for debugging
 * - Supports custom fallback UI
 * - Shows error details in development mode
 */
class ErrorBoundary extends Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            errorInfo: null,
        };
    }

    static getDerivedStateFromError(error: Error): Partial<State> {
        return {
            hasError: true,
            error,
        };
    }

    componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error("ErrorBoundary caught an error:", error, errorInfo);

        this.setState({
            error,
            errorInfo,
        });

        if (this.props.onError) {
            this.props.onError(error, errorInfo);
        }
    }

    handleReload = () => {
        window.location.reload();
    };

    handleReset = () => {
        this.setState({
            hasError: false,
            error: null,
            errorInfo: null,
        });
    };

    render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback;
            }

            return (
                <div className="error-boundary">
                    <div className="error-boundary-content">
                        <div className="error-icon">
                            <AlertTriangle size={48} />
                        </div>

                        <h1 className="error-title">
                            Oops! Something went wrong
                        </h1>

                        <p className="error-description">
                            We're sorry, but something unexpected happened. Your
                            downloads and settings are safe.
                        </p>

                        <div className="error-actions">
                            <button
                                className="error-button primary"
                                onClick={this.handleReload}
                            >
                                <RefreshCw size={18} />
                                Reload Application
                            </button>

                            <button
                                className="error-button secondary"
                                onClick={this.handleReset}
                            >
                                Try Again
                            </button>
                        </div>

                        {import.meta.env.DEV && this.state.error && (
                            <details className="error-details">
                                <summary>Error Details (Dev Mode)</summary>
                                <div className="error-stack">
                                    <strong>Error:</strong>
                                    <pre>{this.state.error.toString()}</pre>

                                    {this.state.errorInfo && (
                                        <>
                                            <strong>Component Stack:</strong>
                                            <pre>
                                                {
                                                    this.state.errorInfo
                                                        .componentStack
                                                }
                                            </pre>
                                        </>
                                    )}
                                </div>
                            </details>
                        )}

                        <p className="error-help">
                            If this problem persists, please{" "}
                            <a
                                href="https://github.com/BurgessTheGamer/ripVID/issues"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="error-link"
                            >
                                report it on GitHub
                            </a>
                        </p>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
