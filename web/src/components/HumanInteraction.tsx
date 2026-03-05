import { useState } from 'react';
import type { TaskStatus, HumanMessage } from '@/types/task';

interface HumanInteractionProps {
  taskId: string;
  isConnected: boolean;
  sendMessage: (message: HumanMessage) => void;
  taskStatus: TaskStatus;
}

export function HumanInteraction({
  taskId,
  isConnected,
  sendMessage,
  taskStatus,
}: HumanInteractionProps) {
  const [message, setMessage] = useState('');
  const [terminalInput, setTerminalInput] = useState('');
  const [terminalId, setTerminalId] = useState('');

  const handleSendMessage = () => {
    if (!message.trim()) return;

    sendMessage({
      type: 'send_message',
      task_id: taskId,
      message: message.trim(),
    });
    setMessage('');
  };

  const handleTerminalInput = () => {
    if (!terminalInput.trim() || !terminalId.trim()) return;

    sendMessage({
      type: 'terminal_input',
      task_id: taskId,
      terminal_id: terminalId.trim(),
      input: terminalInput.trim(),
    });
    setTerminalInput('');
  };

  const handleAbort = () => {
    if (confirm('Are you sure you want to abort this task?')) {
      sendMessage({
        type: 'abort_task',
        task_id: taskId,
      });
    }
  };

  const handleAcceptWork = () => {
    if (confirm('Accept the work and mark task as complete?')) {
      sendMessage({
        type: 'accept_work',
        task_id: taskId,
      });
    }
  };

  const isActive = taskStatus === 'in_progress' || taskStatus === 'iterating';
  const isAwaitingReview = taskStatus === 'awaiting_review';

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="px-6 py-4 border-b border-gray-200">
        <h2 className="text-xl font-bold text-gray-900">Human Interaction</h2>
      </div>

      <div className="p-6 space-y-6">
        {!isConnected && (
          <div className="bg-yellow-50 border border-yellow-200 rounded p-3">
            <p className="text-sm text-yellow-800">
              ⚠️ Not connected to task. Changes may not be sent.
            </p>
          </div>
        )}

        <div>
          <h3 className="font-medium text-gray-900 mb-2">Send Message</h3>
          <textarea
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder="Type a message to the agent..."
            className="w-full border rounded p-2 h-24 resize-none"
            disabled={!isActive}
          />
          <button
            onClick={handleSendMessage}
            disabled={!isActive || !isConnected}
            className="mt-2 w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
          >
            Send Message
          </button>
        </div>

        <div>
          <h3 className="font-medium text-gray-900 mb-2">Terminal Input</h3>
          <input
            type="text"
            value={terminalId}
            onChange={(e) => setTerminalId(e.target.value)}
            placeholder="Terminal ID"
            className="w-full border rounded p-2 mb-2"
            disabled={!isActive}
          />
          <input
            type="text"
            value={terminalInput}
            onChange={(e) => setTerminalInput(e.target.value)}
            placeholder="Command or input..."
            className="w-full border rounded p-2"
            disabled={!isActive}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleTerminalInput();
              }
            }}
          />
          <button
            onClick={handleTerminalInput}
            disabled={!isActive || !isConnected}
            className="mt-2 w-full bg-gray-600 text-white py-2 px-4 rounded hover:bg-gray-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
          >
            Send to Terminal
          </button>
        </div>

        <div className="pt-4 border-t space-y-2">
          {isActive && (
            <button
              onClick={handleAbort}
              disabled={!isConnected}
              className="w-full bg-red-600 text-white py-2 px-4 rounded hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
            >
              ⚠️ Abort Task
            </button>
          )}

          {isAwaitingReview && (
            <button
              onClick={handleAcceptWork}
              disabled={!isConnected}
              className="w-full bg-green-600 text-white py-2 px-4 rounded hover:bg-green-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
            >
              ✓ Accept Work
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
