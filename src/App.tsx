import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Layout } from "@/components/layout";
import { Dashboard, Markets, MarketDetail, Portfolio, Settings } from "@/pages";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Dashboard />} />
          <Route path="markets" element={<Markets />} />
          <Route path="markets/:conditionId" element={<MarketDetail />} />
          <Route path="portfolio" element={<Portfolio />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
