// AIDEV-NOTE: Price chart using Lightweight Charts v5 - supports line and area charts
import { useEffect, useRef, useCallback } from "react";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type AreaData,
  type Time,
  ColorType,
  AreaSeries,
} from "lightweight-charts";
import { cn } from "@/lib/utils";

export interface PriceDataPoint {
  time: Time;
  value: number;
}

interface PriceChartProps {
  data: PriceDataPoint[];
  className?: string;
  height?: number;
  lineColor?: string;
  areaTopColor?: string;
  areaBottomColor?: string;
  showGrid?: boolean;
}

export function PriceChart({
  data,
  className,
  height = 300,
  lineColor = "#2563eb",
  areaTopColor = "rgba(37, 99, 235, 0.4)",
  areaBottomColor = "rgba(37, 99, 235, 0.0)",
  showGrid = true,
}: PriceChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Area"> | null>(null);

  // Initialize chart
  useEffect(() => {
    if (!containerRef.current) return;

    const chart = createChart(containerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#9ca3af",
      },
      grid: {
        vertLines: { visible: showGrid, color: "#27272a" },
        horzLines: { visible: showGrid, color: "#27272a" },
      },
      width: containerRef.current.clientWidth,
      height,
      rightPriceScale: {
        borderColor: "#27272a",
      },
      timeScale: {
        borderColor: "#27272a",
        timeVisible: true,
      },
      crosshair: {
        mode: 1,
        vertLine: {
          color: "#4b5563",
          labelBackgroundColor: "#374151",
        },
        horzLine: {
          color: "#4b5563",
          labelBackgroundColor: "#374151",
        },
      },
    });

    // v5 API: use addSeries with AreaSeries type
    const series = chart.addSeries(AreaSeries, {
      lineColor,
      topColor: areaTopColor,
      bottomColor: areaBottomColor,
      lineWidth: 2,
      priceFormat: {
        type: "custom",
        formatter: (price: number) => `${(price * 100).toFixed(1)}¢`,
      },
    });

    chartRef.current = chart;
    seriesRef.current = series;

    // Handle resize
    const handleResize = () => {
      if (containerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: containerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
    };
  }, [height, lineColor, areaTopColor, areaBottomColor, showGrid]);

  // Update data
  useEffect(() => {
    if (seriesRef.current && data.length > 0) {
      seriesRef.current.setData(data as AreaData<Time>[]);
      chartRef.current?.timeScale().fitContent();
    }
  }, [data]);

  return (
    <div
      ref={containerRef}
      className={cn("w-full", className)}
      style={{ height }}
    />
  );
}

// Hook for managing real-time chart updates
export function useChartUpdates() {
  const priceHistoryRef = useRef<PriceDataPoint[]>([]);

  const addPrice = useCallback((price: number, timestamp?: number) => {
    const time = (timestamp ? timestamp / 1000 : Date.now() / 1000) as Time;
    priceHistoryRef.current.push({ time, value: price });

    // Keep last 1000 points
    if (priceHistoryRef.current.length > 1000) {
      priceHistoryRef.current = priceHistoryRef.current.slice(-1000);
    }

    return [...priceHistoryRef.current];
  }, []);

  const clearHistory = useCallback(() => {
    priceHistoryRef.current = [];
  }, []);

  return { priceHistory: priceHistoryRef.current, addPrice, clearHistory };
}
