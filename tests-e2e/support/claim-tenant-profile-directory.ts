import { readdir, readFile, rm, rmdir } from "node:fs/promises";
import path from "node:path";

import { RELEASE_BUILD } from "./release-build";

/** Fichier de fiche écrit par un scénario, reconnu à son nom et à l'un de ses contenus possibles. */
export interface ScenarioTenantProfileFile {
  readonly fileName: string;
  readonly knownContents: readonly string[];
}

/** Dossier des fiches du build release, réservé à un scénario. */
export interface TenantProfileDirectoryClaim {
  /** Chemin du fichier que le scénario fait écrire à l'app. */
  readonly filePath: string;
  /** Retire le fichier du scénario, et le dossier si le scénario l'a fait créer. */
  readonly release: () => Promise<void>;
}

function isMissingPath(err: unknown): boolean {
  return err instanceof Error && "code" in err && err.code === "ENOENT";
}

async function listEntries(directory: string): Promise<readonly string[] | null> {
  try {
    return await readdir(directory);
  } catch (err: unknown) {
    if (isMissingPath(err)) {
      return null;
    }
    throw err;
  }
}

async function isScenarioEntry(options: {
  readonly entry: string;
  readonly file: ScenarioTenantProfileFile;
}): Promise<boolean> {
  if (options.entry !== options.file.fileName) {
    return false;
  }
  const contents = await readFile(
    path.join(RELEASE_BUILD.tenantProfileDirectory, options.entry),
    "utf8",
  );
  return options.file.knownContents.includes(contents);
}

/**
 * Retire le fichier du scénario, après avoir vérifié que le dossier ne contient rien
 * d'autre : le build release le partage avec l'usage réel de l'app, dont les fiches ne
 * doivent jamais être effacées. Indique si le dossier existe.
 */
async function removeScenarioFile(file: ScenarioTenantProfileFile): Promise<boolean> {
  const directory = RELEASE_BUILD.tenantProfileDirectory;
  const entries = await listEntries(directory);
  if (entries === null) {
    return false;
  }
  const ownership = await Promise.all(
    entries.map(async (entry) => ({ entry, isOwned: await isScenarioEntry({ entry, file }) })),
  );
  const foreignEntries = ownership.filter(({ isOwned }) => !isOwned).map(({ entry }) => entry);
  if (foreignEntries.length > 0) {
    throw new Error(
      `[E2E_FOREIGN_TENANT_PROFILES] ${directory} holds entries this scenario did not write (${foreignEntries.join(", ")}): move them out of the release build before running the e2e tests`,
    );
  }
  await rm(path.join(directory, file.fileName), { force: true });
  return true;
}

/**
 * Réserve le dossier `fiches/` du build release à un scénario : refuse un dossier qui
 * contient autre chose que le fichier du scénario, et retire celui qu'un scénario
 * interrompu aurait laissé.
 */
export async function claimTenantProfileDirectory(
  file: ScenarioTenantProfileFile,
): Promise<TenantProfileDirectoryClaim> {
  const isDirectoryPresentBefore = await removeScenarioFile(file);
  return {
    filePath: path.join(RELEASE_BUILD.tenantProfileDirectory, file.fileName),
    release: async (): Promise<void> => {
      const isDirectoryPresent = await removeScenarioFile(file);
      if (isDirectoryPresent && !isDirectoryPresentBefore) {
        await rmdir(RELEASE_BUILD.tenantProfileDirectory);
      }
    },
  };
}
