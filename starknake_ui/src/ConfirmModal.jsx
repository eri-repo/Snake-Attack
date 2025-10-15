import React from 'react';
import './ConfirmModal.css';

export default function ConfirmModal({ message, onConfirm, onCancel }) {
  return (
    <div className="confirm-modal-overlay">
      <div className="confirm-modal">
        <div className="confirm-modal-content">
          <p className="confirm-modal-message">{message}</p>
          <div className="confirm-modal-actions">
            <button className="confirm-modal-btn confirm-modal-cancel" onClick={onCancel}>
              Cancel
            </button>
            <button className="confirm-modal-btn confirm-modal-confirm" onClick={onConfirm}>
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
