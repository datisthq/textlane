import * as plausible from "@plausible-analytics/tracker"
import { StartClient } from "@tanstack/react-start/client"
import { StrictMode } from "react"
import { hydrateRoot } from "react-dom/client"
import { config } from "livemark:virtual"

if (config.site && location.hostname !== "localhost") {
  plausible.init({
    domain: new URL(config.site).hostname,
    outboundLinks: true,
  })
}

hydrateRoot(
  document,
  <StrictMode>
    <StartClient />
  </StrictMode>,
)
