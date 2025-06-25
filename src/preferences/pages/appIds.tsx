import { createCustomPreferencePage } from "../helpers";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { useStore } from "../../utilities/StoreContext";
import { Button, Typography } from "@mui/joy";

type AppId = {
  app_id_id: string;
  identifier: string;
  name: string;
  features: Record<string, any>;
  expiration_date: Date;
};

type AppIdsResponse = {
  app_ids: AppId[];
  max_quantity: number;
  available_quantity: number;
};

const AppIdsComponent = () => {
  const [ids, setIds] = useState<AppId[]>([]);
  const [maxQuantity, setMaxQuantity] = useState<number | null>(null);
  const [availableQuantity, setAvailableQuantity] = useState<number | null>(
    null
  );
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [anisetteServer] = useStore<string>(
    "apple-id/anisette-server",
    "ani.sidestore.io"
  );
  const [canDelete] = useStore<boolean>("developer/delete-app-ids", false);

  useEffect(() => {
    let fetch = async () => {
      let ids = await invoke<AppIdsResponse>("list_app_ids", {
        anisetteServer,
      });
      setIds(ids.app_ids);
      setMaxQuantity(ids.max_quantity);
      setAvailableQuantity(ids.available_quantity);
      setLoading(false);
    };
    fetch().catch((e) => {
      console.error("Failed to fetch certificates:", e);
      setError(
        "Failed to load certificates: " + e + "\nPlease try again later."
      );
      setLoading(false);
    });
  }, []);

  if (loading) {
    return <div>Loading App IDs...</div>;
  }
  if (error) {
    return <div className="error">{error}</div>;
  }

  return (
    <div>
      <div style={{ marginBottom: "var(--padding-lg)" }}>
        You have {availableQuantity}/{maxQuantity} App IDs available.
      </div>
      <ul style={{ margin: 0, padding: 0, listStyleType: "none" }}>
        {ids.map((id, idx) => (
          <li
            key={id.app_id_id}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "var(--padding-md)",
              borderBottom:
                idx < ids.length - 1
                  ? "1px solid var(--joy-palette-neutral-800)"
                  : "none",
            }}
          >
            {canDelete && (
              <Button
                variant="soft"
                color="warning"
                onClick={async () => {
                  try {
                    await invoke("delete_app_id", {
                      anisetteServer,
                      appIdId: id.app_id_id,
                    });
                    setIds((prev) =>
                      prev.filter((c) => c.app_id_id !== id.app_id_id)
                    );
                  } catch (e) {
                    console.error("Failed to delete app ID:", e);
                    alert(
                      "Failed to revoke app ID: " +
                        e +
                        "\nPlease try again later."
                    );
                  }
                }}
              >
                Delete
              </Button>
            )}
            <div>
              <div>
                {id.name}: {id.app_id_id}
              </div>
              <Typography
                level="body-xs"
                sx={{ color: "var(--joy-palette-neutral-500)" }}
              >
                {id.identifier}
              </Typography>
            </div>
            <div style={{ flexGrow: 1, textAlign: "right" }}>
              <Typography level="body-sm">
                Expires {new Date(id.expiration_date).toLocaleDateString()}
              </Typography>
            </div>
          </li>
        ))}
        {ids.length === 0 && <li>No App IDs found.</li>}
      </ul>
    </div>
  );
};

export const appIdsPage = createCustomPreferencePage(
  "appids",
  "App IDs",
  AppIdsComponent,
  {
    description:
      "Free developer accounts have a limit of 10 App IDs. You cannot delete App IDs, but they will expire after a week.",
    category: "apple",
  }
);
