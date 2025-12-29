import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "@/components/layout";
import { WebSocketProvider } from "@/components/providers/WebSocketProvider";
import { Dashboard, Markets, MarketDetail, Portfolio, Settings } from "@/pages";

function App() {
  return (
    <WebSocketProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Dashboard />} />
            <Route path="markets" element={<Markets />} />
            <Route path="markets/:marketId" element={<MarketDetail />} />
            <Route path="portfolio" element={<Portfolio />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </WebSocketProvider>
  );
}

export default App;
