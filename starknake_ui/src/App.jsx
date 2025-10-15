import React, { useState } from "react";
import GameLoader from "./GameLoader";
import Landing from "./Landing";
import Dashboard from "./Dashboard";
import ConfirmModal from "./ConfirmModal";

export default function App() {
  const [view, setView] = useState(() => {
    const savedView = sessionStorage.getItem('gameView');
    return savedView || 'landing';
  });
  const [playerDetails, setPlayerDetails] = useState(() => {
    const savedDetails = sessionStorage.getItem('playerDetails');
    return savedDetails ? JSON.parse(savedDetails) : null;
  });
  const [showExitConfirm, setShowExitConfirm] = useState(false);
  console.log("App mounted, view =", view);

  const handleGoToDashboard = (details) => {
    setPlayerDetails(details);
    sessionStorage.setItem('playerDetails', JSON.stringify(details));
    sessionStorage.setItem('gameView', 'dashboard');
    setView("dashboard");
  };

  const handleBackToLanding = () => {
    setPlayerDetails(null);
    sessionStorage.removeItem('playerDetails');
    sessionStorage.setItem('gameView', 'landing');
    setView("landing");
  };

  const handleUpdatePlayerDetails = (details) => {
    setPlayerDetails(details);
    sessionStorage.setItem('playerDetails', JSON.stringify(details));
  };

  const handleExitGame = () => {
    try {
      if (
        window.s_oGame &&
        typeof window.s_oGame.unload === "function"
      ) {
        window.s_oGame.unload();
      }
      if (
        window.s_oMain &&
        typeof window.s_oMain.stopUpdate === "function"
      ) {
        window.s_oMain.stopUpdate();
      }
    } catch (e) {
      // ignore cleanup errors
    }

    // Save the target view before reload
    if (playerDetails?.walletAddress) {
      sessionStorage.setItem('gameView', 'dashboard');
    } else {
      sessionStorage.setItem('gameView', 'landing');
    }

    // Reload to clean up game state
    window.location.reload();
  };

  return (
    <div style={{ width: "100%", height: "100%" }}>
      {view === "landing" && (
        <Landing
          onGoToDashboard={handleGoToDashboard}
          onPlayGuest={() => {
            sessionStorage.setItem('gameView', 'game');
            setView("game");
          }}
        />
      )}

      {view === "dashboard" && (
        <Dashboard
          playerDetails={playerDetails}
          onPlay={() => {
            sessionStorage.setItem('gameView', 'game');
            setView("game");
          }}
          onBack={handleBackToLanding}
          onUpdatePlayerDetails={handleUpdatePlayerDetails}
        />
      )}

      {view === "game" && (
        <>
          {/* Back to menu overlay button */}
          <button
            onClick={() => setShowExitConfirm(true)}
            style={{
              position: "fixed",
              left: 12,
              top: 12,
              zIndex: 9999,
              padding: "8px 12px",
              background: "#222",
              color: "#fff",
              borderRadius: 6,
              border: "none",
              cursor: "pointer",
              opacity: 0.9,
            }}
          >
            Back to Menu
          </button>

          {showExitConfirm && (
            <ConfirmModal
              message="Return to menu? Your current game progress will be lost."
              onConfirm={handleExitGame}
              onCancel={() => setShowExitConfirm(false)}
            />
          )}

          <canvas
            id="canvas"
            className="ani_hack"
            width="1360"
            height="768"
          ></canvas>
          <div
            data-orientation="landscape"
            className="orientation-msg-container"
          >
            <p className="orientation-msg-text">Please rotate your device</p>
          </div>
          <div
            id="block_game"
            style={{
              position: "fixed",
              backgroundColor: "transparent",
              top: 0,
              left: 0,
              width: "100%",
              height: "100%",
              display: "none",
            }}
          ></div>
          <GameLoader
            playerDetails={playerDetails}
            onUpdatePlayerDetails={handleUpdatePlayerDetails}
          />
        </>
      )}
    </div>
  );
}
