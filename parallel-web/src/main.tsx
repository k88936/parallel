import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import "./index.css"
import Theme, {ThemeProvider} from "@jetbrains/ring-ui-built/components/global/theme";
import {App} from "./App.tsx";

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <ThemeProvider theme={Theme.DARK}>
            <div style={{
                minHeight: "100vh",
                borderRadius: "var(--ring-border-radius)",
                backgroundColor: "var(--ring-content-background-color)",
            }}>
                <App/>
            </div>
        </ThemeProvider>
    </StrictMode>,
)
