import React, { useState, useEffect } from 'react'
import { RpcProvider, Contract } from 'starknet'
import { connect } from 'starknetkit'
import { contractABI } from './abi'
import './dashboard.css'

const CONTRACT_ADDRESS =
  import.meta.env.VITE_CONTRACT ||
  "0x3060854ecff13fd7f72caf971475823ff457fed1de616e70cd5342ffa345d88";
const PROVIDER_URL =
  import.meta.env.VITE_SEPOLIA_URL ||
  "https://free-rpc.nethermind.io/sepolia-juno";
const BACKEND_URL = import.meta.env.VITE_APP_URL || "http://127.0.0.1:8080";

const PROVIDER = new RpcProvider({ nodeUrl: PROVIDER_URL });

export default function Dashboard({ playerDetails, onPlay, onBack, onUpdatePlayerDetails }) {
  const [isEditingUsername, setIsEditingUsername] = useState(false);
  const [newUsername, setNewUsername] = useState('');
  const [updateLoading, setUpdateLoading] = useState(false);
  const [error, setError] = useState(null);
  const [isRefreshing, setIsRefreshing] = useState(false);

  useEffect(() => {
    const fetchLatestUserData = async () => {
      if (!playerDetails?.walletAddress) return;

      setIsRefreshing(true);
      try {
        const response = await fetch(`${BACKEND_URL}/users/${playerDetails.walletAddress}`);
        if (response.ok) {
          const userData = await response.json();
          console.log("Refreshed user data:", userData);

          const updatedDetails = {
            username: userData.username,
            position: userData.position,
            walletAddress: playerDetails.walletAddress,
            score: userData.highest_score.toString(),
          };

          onUpdatePlayerDetails(updatedDetails);
        }
      } catch (error) {
        console.error("Error refreshing user data:", error);
      } finally {
        setIsRefreshing(false);
      }
    };

    fetchLatestUserData();
  }, []);

  const updateUsername = async () => {
    console.log("playerDetails:", playerDetails);
    console.log("walletAddress:", playerDetails?.walletAddress);

    if (!playerDetails?.walletAddress) {
      setError("Wallet address not found");
      return;
    }

    const trimmedUsername = newUsername.trim();

    if (!trimmedUsername || trimmedUsername.length === 0 || trimmedUsername.length > 50) {
      setError("Please enter a valid username (1-50 characters)");
      return;
    }

    if (!trimmedUsername.match(/^[a-zA-Z0-9_]+$/)) {
      setError("Username must contain only alphanumeric characters or underscores");
      return;
    }

    setUpdateLoading(true);
    setError(null);
    try {
      const checkResponse = await fetch(`${BACKEND_URL}/check_user`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          wallet_address: playerDetails.walletAddress,
          username: trimmedUsername,
        }),
      });

      if (!checkResponse.ok) {
        const errorData = await checkResponse.json();
        throw new Error(
          `Failed to check user: ${errorData.error || "Unknown error"}`
        );
      }

      const userData = await checkResponse.json();
      console.log("Check user response:", userData);

      if (userData.updated === false) {
        try {
          const { wallet } = await connect({
            modalMode: "neverAsk",
            modalTheme: "dark",
          });

          if (!wallet?.isConnected) {
            throw new Error("Wallet not connected");
          }

          console.log("Wallet connected:", wallet);

          const contract = new Contract(
            contractABI,
            CONTRACT_ADDRESS,
            wallet.account
          );
          console.log("Contract instance:", contract);

          const res = await contract.player_update_username(trimmedUsername);
          const txHash = res?.transaction_hash;
          if (!txHash) {
            throw new Error("No transaction hash returned from contract call");
          }
          const txResult = await PROVIDER.waitForTransaction(txHash);
          console.log("Contract transaction:", txResult);
        } catch (contractError) {
          console.error("Contract call failed:", contractError);
          throw new Error(
            `Failed to update username on contract: ${contractError.message}`
          );
        }
      } else {
        console.log("User already updated, skipping contract call");
      }

      const updateResponse = await fetch(`${BACKEND_URL}/update_user`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          wallet_address: playerDetails.walletAddress,
          new_username: trimmedUsername,
        }),
      });

      if (!updateResponse.ok) {
        const errorData = await updateResponse.json();
        throw new Error(
          `Failed to update username: ${errorData.error || "Unknown error"}`
        );
      }

      const updateData = await updateResponse.json();
      console.log("Update response:", updateData);

      onUpdatePlayerDetails({
        username: updateData.username,
        position: updateData.position,
        walletAddress: playerDetails.walletAddress,
        score: updateData.highest_score.toString(),
      });

      setIsEditingUsername(false);
      setNewUsername('');
      setError(null);
    } catch (error) {
      const message = error.message || "Unknown error";
      setError(message);
      console.error("Update error:", message);
    } finally {
      setUpdateLoading(false);
    }
  };

  const handleCancelEdit = () => {
    setIsEditingUsername(false);
    setNewUsername('');
    setError(null);
  };

  return (
    <div className="dashboard-root">
      <div className="dashboard-container">
        <h2 className="dashboard-title">Player Dashboard</h2>
        {playerDetails && (
          <div className="dashboard-info">
            <div className="info-card">
              <div className="info-label">Username</div>
              {!isEditingUsername ? (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div className="info-value">{playerDetails.username}</div>
                  <button
                    onClick={() => setIsEditingUsername(true)}
                    className="btn btn-secondary btn-small"
                  >
                    Edit
                  </button>
                </div>
              ) : (
                <div className="edit-controls">
                  <div className="username-edit-container">
                    <input
                      type="text"
                      value={newUsername}
                      onChange={(e) => setNewUsername(e.target.value)}
                      placeholder="Enter new username"
                      className="edit-input"
                      disabled={updateLoading}
                      autoFocus
                    />
                    <button
                      onClick={updateUsername}
                      disabled={updateLoading}
                      className="btn btn-primary btn-small"
                    >
                      {updateLoading ? 'Saving...' : 'Save'}
                    </button>
                    <button
                      onClick={handleCancelEdit}
                      disabled={updateLoading}
                      className="btn btn-secondary btn-small"
                    >
                      Cancel
                    </button>
                  </div>
                  {error && (
                    <div className="error-text">
                      {error}
                    </div>
                  )}
                </div>
              )}
            </div>

            <div className="info-card">
              <div className="info-label">Wallet Address</div>
              <div className="info-value" style={{ fontSize: '14px' }}>
                {playerDetails.walletAddress}
              </div>
            </div>

            <div className="info-card">
              <div className="info-label">High Score</div>
              <div className="score-highlight">
                {playerDetails.score}
              </div>
            </div>
          </div>
        )}
        <div className="dashboard-actions">
          <button onClick={onPlay} className="btn btn-primary">
            Play Game
          </button>
          <button onClick={onBack} className="btn btn-secondary">
            Back to Menu
          </button>
        </div>
      </div>
    </div>
  )
}
