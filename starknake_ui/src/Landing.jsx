import React, { useState, useEffect } from "react";
import { connect } from "starknetkit";
import { RpcProvider, Contract } from "starknet";
import { contractABI } from "./abi";
import "./landing.css";

const CONTRACT_ADDRESS =
  import.meta.env.VITE_CONTRACT ||
  "0x3060854ecff13fd7f72caf971475823ff457fed1de616e70cd5342ffa345d88";
const PROVIDER_URL =
  import.meta.env.VITE_SEPOLIA_URL ||
  "https://free-rpc.nethermind.io/sepolia-juno";
const BACKEND_URL = import.meta.env.VITE_APP_URL || "http://127.0.0.1:8080";

const PROVIDER = new RpcProvider({ nodeUrl: PROVIDER_URL });

function getFriendlyErrorMessage(error) {
  const message = error && error.message ? error.message : String(error);
  if (message.includes("user rejected") || message.includes("authorize")) {
    return "Connection cancelled. Please approve the connection in your wallet.";
  }
  if (
    message.includes("No StarkNet") ||
    message.toLowerCase().includes("detected")
  ) {
    return "No StarkNet wallet found. Please install Argent X or Braavos and refresh the page.";
  }
  return `Wallet connection failed: ${message}`;
}

export default function Landing({ onGoToDashboard, onPlayGuest }) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [leaderboard, setLeaderboard] = useState([]);
  const [leaderboardLoading, setLeaderboardLoading] = useState(true);

  useEffect(() => {
    const fetchLeaderboard = async () => {
      try {
        const response = await fetch(`${BACKEND_URL}/leaderboard`);
        if (response.ok) {
          const data = await response.json();
          console.log("Leaderboard data:", data);
          if (Array.isArray(data)) {
            setLeaderboard(data);
          } else {
            console.error("Leaderboard data is not an array:", data);
            setLeaderboard([]);
          }
        } else {
          console.error("Failed to fetch leaderboard, status:", response.status);
          setLeaderboard([]);
        }
      } catch (error) {
        console.error("Error fetching leaderboard:", error);
        setLeaderboard([]);
      } finally {
        setLeaderboardLoading(false);
      }
    };

    fetchLeaderboard();
    const interval = setInterval(fetchLeaderboard, 5000);

    return () => clearInterval(interval);
  }, []);

  const connectWallet = async () => {
    setLoading(true);
    setError(null);

    try {
      console.log("Connecting to wallet...");
      if (typeof window === "undefined" || !window.starknet) {
        throw new Error(
          "No StarkNet wallet detected. Please install Argent X or Braavos."
        );
      }

      const { wallet } = await connect({
        provider: PROVIDER,
      });

      console.log("Wallet object:", wallet);

      if (
        wallet &&
        wallet.isConnected &&
        wallet.account &&
        wallet.selectedAddress
      ) {
        try {
          console.log("Sending wallet address:", wallet.selectedAddress);
          const response = await fetch(`${BACKEND_URL}/create_user`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({ wallet_address: wallet.selectedAddress }),
          });

          if (!response.ok) {
            const errorData = await response.json();
            throw new Error(
              `Failed to fetch user data: ${errorData.error || "Unknown error"}`
            );
          }

          const data = await response.json();
          console.log("Backend response:", data);
          console.log("Status response:", data.status);

          if (data.status === "New") {
            try {
              console.log("Contract ABI:", contractABI);

              const contract = new Contract(
                contractABI,
                CONTRACT_ADDRESS,
                wallet.account
              );
              console.log("Contract instance:", contract);

              const res = await contract.player_registers(
                wallet.selectedAddress,
                data.username
              );
              const txHash = res?.transaction_hash;
              if (!txHash) {
                throw new Error(
                  "No transaction hash returned from contract call"
                );
              }
              const txResult = await PROVIDER.waitForTransaction(txHash);
              console.log("Contract transaction:", txResult);
            } catch (contractError) {
              console.error("Contract call failed:", contractError);

              const deleteResponse = await fetch(`${BACKEND_URL}/delete_user`, {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  wallet_address: wallet.selectedAddress,
                }),
              });

              if (!deleteResponse.ok) {
                const errorData = await deleteResponse.json();
                throw new Error(
                  `Failed to delete user data from database: ${
                    errorData.error || "Unknown error"
                  }`
                );
              }

              throw new Error(
                `Failed to register player on contract: ${contractError.message}`
              );
            }
          } else {
            console.log("Existing user, skipping contract registration");
          }

          onGoToDashboard({
            username: data.username,
            position: data.position,
            walletAddress: wallet.selectedAddress,
            score: data.highest_score.toString(),
          });
        } catch (fetchError) {
          console.error("Backend or contract error:", fetchError);
          throw fetchError;
        }
      } else {
        throw new Error("Failed to connect wallet");
      }
    } catch (error) {
      const message = getFriendlyErrorMessage(error);
      setError(message);
      console.error("Connection error:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="landing-root">
      <div className="landing-container">
        <div className="landing-content">
          <img src="/sprites/logo.png" alt="logo" className="landing-logo" />
          <p className="landing-sub">
            Classic snake action with modern twists. Survive, eat, grow, and
            conquer.
          </p>
          {error && (
            <div
              className="error-message"
              style={{
                color: "#ff6b6b",
                background: "rgba(255, 107, 107, 0.1)",
                padding: "14px",
                borderRadius: "12px",
                marginBottom: "20px",
                maxWidth: "400px",
                margin: "0 auto 20px",
              }}
            >
              {error}
            </div>
          )}
          <div
            style={{
              display: "flex",
              gap: 12,
              justifyContent: "center",
              marginTop: 16,
            }}
          >
            <button className="landing-play" onClick={onPlayGuest}>
              Play as Guest
            </button>
            <button
              className="landing-play"
              onClick={connectWallet}
              disabled={loading}
            >
              {loading ? "Connecting..." : "Connect Wallet"}
            </button>
          </div>
          <div className="landing-footer">
            Use arrow keys or tap sides on mobile to steer
          </div>
        </div>

        <div className="leaderboard-panel">
          <h2 className="leaderboard-title">Top Players</h2>
          {leaderboardLoading ? (
            <div className="leaderboard-loading">Loading...</div>
          ) : leaderboard.length === 0 ? (
            <div className="leaderboard-empty">No players yet</div>
          ) : (
            <div className="leaderboard-list">
              {leaderboard.slice(0, 10).map((player, index) => (
                <div key={player.wallet_address || index} className="leaderboard-item">
                  <div className="leaderboard-rank">#{index + 1}</div>
                  <div className="leaderboard-info">
                    <div className="leaderboard-username">
                      {player.username || 'Unknown'}
                    </div>
                    <div className="leaderboard-wallet">
                      {player.walletAddress
                        ? `${player.walletAddress.slice(0, 6)}...${player.walletAddress.slice(-4)}`
                        : 'N/A'
                      }
                    </div>
                    <div className="leaderboard-games">
                      {player.games_played || 0} {player.games_played === 1 ? 'game' : 'games'}
                    </div>
                  </div>
                  <div className="leaderboard-score">{player.highest_score || 0}</div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
