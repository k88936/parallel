import { useEffect, useRef } from 'react';

interface AgentOutputProps {
  output: string[];
}

export function AgentOutput({ output }: AgentOutputProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [output]);

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="px-6 py-4 border-b border-gray-200">
        <h2 className="text-xl font-bold text-gray-900">Agent Output</h2>
      </div>

      <div
        ref={containerRef}
        className="bg-gray-900 text-gray-100 p-4 h-96 overflow-y-auto font-mono text-sm"
      >
        {output.length === 0 ? (
          <div className="text-gray-500 italic">Waiting for agent output...</div>
        ) : (
          output.map((line, index) => (
            <div key={index} className="whitespace-pre-wrap">
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
