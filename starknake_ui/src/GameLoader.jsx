import { useEffect, useRef } from 'react'
import { Contract } from 'starknet'
import { connect } from 'starknetkit'
import { contractABI } from './abi'

const BACKEND_URL = import.meta.env.VITE_APP_URL || "http://127.0.0.1:8080";
const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT || "0x3060854ecff13fd7f72caf971475823ff457fed1de616e70cd5342ffa345d88";

// List of legacy scripts to load in order (includes core libs first)
const legacyScripts = [
  '/js/jquery-3.1.1.min.js',
  '/js/createjs.min.js',
  '/js/screenfull.js',
  '/js/howler.min.js',
  '/js/platform.js',
  '/js/ios_fullscreen.js',
  '/js/ctl_utils.js',
  '/js/sprite_lib.js',
  '/js/settings.js',
  '/js/CLang.js',
  '/js/CPreloader.js',
  '/js/CMain.js',
  '/js/CTextButton.js',
  '/js/CToggle.js',
  '/js/CGfxButton.js',
  '/js/CCreditsPanel.js',
  '/js/CMenu.js',
  '/js/CGame.js',
  '/js/CInterface.js',
  '/js/CHelpPanel.js',
  '/js/CEndPanel.js',
  '/js/CSnake.js',
  '/js/CSingleQueue.js',
  '/js/CVector2.js',
  '/js/CEdges.js',
  '/js/CEdge.js',
  '/js/CManageFoods.js',
  '/js/CFood.js',
  '/js/CControlAiSnakes.js',
  '/js/CSubAISnake.js',
  '/js/CManageSections.js',
  '/js/CSection.js',
  '/js/CPause.js',
  '/js/CAreYouSurePanel.js',
  '/js/CBackground.js',
  '/js/CAnimMenu.js',
  '/js/CLogo.js',
  '/js/CAnimHelp.js'
]

function loadScript(src){
  return new Promise((resolve, reject) => {
    const s = document.createElement('script')
    s.src = src
    s.async = false
    s.onload = () => resolve(src)
    s.onerror = (e) => reject(e)
    document.body.appendChild(s)
  })
}

export default function GameLoader({ playerDetails, onUpdatePlayerDetails }){
  const gamesPlayedRef = useRef(0);

  useEffect(() => {
    // Ensure jQuery, createjs and howler are already loaded from public/index.html
    const loadAll = async () => {
      try{
        for(const s of legacyScripts){
          await loadScript(s)
        }

        // Call the same initialization that original index.html did on document ready
        // The original used: var oMain = new CMain({...}); and $(oMain).on(...)
        // We'll wait for jQuery to be available and then run the same code.
        if(window.jQuery){
          window.$(document).ready(function () {
            // Use the same options as original index.html
            var oMain = new window.CMain({
              hero_rotation_speed: 10,
              hero_speed_up: 15,
              hero_speed: 10,
              snakes_AI_speed: [10, 10, 10, 10],
              food_score: [1],
              fullscreen:true,
              check_orientation:true
            });

            // Reattach event handlers to parent if ctl-arcade is present
            window.$(oMain).on("start_session", function (evt) {
                if (getParamValue('ctl-arcade') === "true") {
                    parent.__ctlArcadeStartSession();
                }
            });

            // Listen for save_score event and send to smart contract then backend
            window.$(oMain).on("save_score", async function (evt, score) {
                console.log("Game over! Points earned:", score);

                if (!playerDetails?.walletAddress) {
                  console.log("No wallet address, skipping score update");
                  return;
                }

                gamesPlayedRef.current += 1;

                try {
                  // First, send to smart contract using player's wallet
                  if (playerDetails?.walletAddress) {
                    console.log("Sending score to smart contract...");
                    try {
                      // Reconnect wallet to get account object
                      const { wallet } = await connect({
                        modalMode: 'neverAsk',
                        modalTheme: 'dark'
                      });

                      if (!wallet?.isConnected || !wallet?.account) {
                        console.log("Wallet not connected, skipping contract call");
                        throw new Error("Wallet not connected");
                      }

                      const contract = new Contract(
                        contractABI,
                        CONTRACT_ADDRESS,
                        wallet.account
                      );

                      const res = await contract.update_player_score(
                        playerDetails.walletAddress,
                        score
                      );

                      console.log("Smart contract transaction:", res?.transaction_hash);

                      // Wait for transaction to be accepted
                      if (res?.transaction_hash) {
                        console.log("Waiting for transaction confirmation...");
                        await wallet.account.provider.waitForTransaction(res.transaction_hash);
                        console.log("Transaction confirmed!");
                      }
                    } catch (contractError) {
                      console.error("Smart contract error:", contractError);
                      // Continue to backend even if contract call fails
                    }
                  }

                  // Then send to backend
                  const response = await fetch(`${BACKEND_URL}/update_score`, {
                    method: "PUT",
                    headers: {
                      "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                      wallet_address: playerDetails.walletAddress,
                      score: score,
                      games_played: gamesPlayedRef.current,
                    }),
                  });

                  if (!response.ok) {
                    const errorData = await response.json();
                    console.error("Failed to update score:", errorData);
                  } else {
                    const data = await response.json();
                    console.log("Score updated successfully:", data);

                    // Fetch updated user data after score update
                    try {
                      const userResponse = await fetch(`${BACKEND_URL}/users/${playerDetails.walletAddress}`);
                      if (userResponse.ok) {
                        const userData = await userResponse.json();
                        console.log("Fetched updated user data:", userData);

                        // Update player details with new score
                        const updatedDetails = {
                          username: userData.username,
                          position: userData.position,
                          walletAddress: playerDetails.walletAddress,
                          score: userData.highest_score.toString(),
                        };

                        if (onUpdatePlayerDetails) {
                          onUpdatePlayerDetails(updatedDetails);
                        }
                      }
                    } catch (fetchError) {
                      console.error("Error fetching updated user data:", fetchError);
                    }
                  }
                } catch (error) {
                  console.error("Error updating score:", error);
                }
            });

            // other event wiring is optional; game will run without arcade wrapper
          })
        } else {
          console.warn('jQuery not found; cannot auto-initialize game');
        }

      }catch(e){
        console.error('Failed loading legacy scripts', e)
      }
    }

    loadAll()
  }, [])

  return null
}
