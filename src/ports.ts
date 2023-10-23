import { invoke } from "@tauri-apps/api";
import {
  emit,
  listen as tauriListen,
  once,
  UnlistenFn,
} from "@tauri-apps/api/event";
import type { Attendant } from "../src-tauri/bindings/Attendant";
import type { RawAttendant } from "../src-tauri/bindings/RawAttendant";
import type { AttendantChecks } from "../src-tauri/bindings/AttendantChecks";
import { WatchStopHandle } from "vue";
import { onKeyStroke } from "@vueuse/core";

interface PortInfo {
  port_name: string;
  port_type:
    | string
    | {
        [type: string]: {
          manufacturer: string;
          pid: number;
          product: string;
          serial_number: string;
          vid: number;
        };
      };
}

type ComputedPortInfo = PortInfo | typeof KEYBOARD_PORT_KEY;

export const KEYBOARD_PORT_KEY = "keyboard";

class Port {
  private _info: ComputedPortInfo;
  private watcher_unlisten: UnlistenFn | WatchStopHandle | undefined;

  constructor(info: ComputedPortInfo) {
    this._info = info;
  }

  get info() {
    return this._info;
  }

  get port_type() {
    if (this._info === "keyboard") return;
    return {
      _type:
        typeof this._info.port_type === "string"
          ? this._info.port_type
          : Object.keys(this._info.port_type)[0],
      ...(typeof this._info.port_type === "string"
        ? {}
        : Object.values(this._info.port_type)[0]),
    };
  }

  public install(input: Ref<string>) {
    if (this._info === "keyboard") {
      onKeyStroke(
        ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
        (e) => {
          e.preventDefault();
          if (PortService.scanning) input.value += e.key;
        },
        { dedupe: true }
      );
      onKeyStroke(["Delete", "Backspace"], (e) => {
        e.preventDefault();
        if (input.value.length > 0 && PortService.scanning)
          input.value = input.value.slice(0, input.value.length - 1);
      });
    } else {
      invoke("start_scan", { portName: this._info.port_name });
    }
    PortService.scanning = true;
  }

  public stop() {
    return new Promise<undefined | true>((resolve) => {
      PortService.scanning = false;
      if (this._info === "keyboard") resolve(undefined);
      emit("close_scan");
      once("scan_closed", ({ payload }) => resolve(payload as true));
    });
  }

  public restart() {
    if (this._info !== "keyboard")
      return invoke("start_scan", { portName: this._info.port_name });
  }

  public async listen(
    userData: Ref<Attendant[] | undefined>,
    input: Ref<string>,
    scanned: Ref<
      {
        data: Partial<RawAttendant>;
        new: boolean;
        checks: AttendantChecks | {};
        time: Date;
        dialogOpen: boolean;
      }[]
    >
  ) {
    const pushScan = (readInput: string) => {
      if (!PortService.scanning) return;

      const fdUser = userData.value?.find((usr) => usr.raw.tui === readInput);

      if (fdUser) {
        scanned.value.push({
          data: fdUser.raw,
          checks: fdUser.checks,
          new: false,
          time: new Date(),
          dialogOpen: false,
        });
      } else {
        scanned.value.push({
          data: {
            tui: readInput,
          },
          checks: {},
          new: true,
          time: new Date(),
          dialogOpen: false,
        });
      }
    };

    if (this._info === "keyboard")
      this.watcher_unlisten = watch(input, () => {
        if (input.value.length !== 7) {
          return;
        }

        const readInput = input.value;
        input.value = "";

        pushScan(readInput);
      });
    else
      this.watcher_unlisten = await tauriListen<string>(
        "id_scanned",
        ({ payload }) => {
          if (!payload) return;

          pushScan(payload.slice(0, 7));
        }
      );
  }

  public unlisten() {
    if (this.watcher_unlisten) {
      this.watcher_unlisten();
    }
  }
}

const ports: Ref<Port[]> = ref([new Port("keyboard")]);
const scanning = ref(false);

export class PortService {
  static get scanning() {
    return scanning.value;
  }

  static set scanning(val: boolean) {
    scanning.value = val;
  }

  static useScanning() {
    return scanning;
  }

  static get ports() {
    return ports.value;
  }

  static get usePorts() {
    return ports;
  }

  public static async fetchPorts(cmd: string | undefined = "get_serial_ports") {
    this.usePorts.value.push(
      ...((await invoke(cmd)) as PortInfo[]).map((info) => new Port(info))
    );
  }
}
