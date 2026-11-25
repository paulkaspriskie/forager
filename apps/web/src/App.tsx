
import { useEffect, useState } from "react";

export default function App() {
  const [data, setData] = useState<unknown>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch("http://127.0.0.1:3000/api/fundamentals/AAPL/ratios")
      .then((response) => {
        if (!response.ok) {
          throw new Error(`HTTP error: ${response.status}`);
        }

        return response.json();
      })
      .then((result) => {
        setData(result);
      })
      .catch((err: unknown) => {
        setError(err instanceof Error ? err.message : "Request failed");
      })
      .finally(() => {
        setLoading(false);
      });
  }, []);

  return (
    <main>
      <h1>Forager</h1>

      {loading && <p>Loading...</p>}

      {error && <p>Error: {error}</p>}

      {Array.isArray(data) &&
        data.map((item, index) => (
          <div key={index}>
            <p>P/E Ratio: {item.priceToEarningsRatio}</p>
            <p>D/E Ratio: {item.debtToEquityRatio}</p>
          </div>
        ))}
    </main>
  );
}
