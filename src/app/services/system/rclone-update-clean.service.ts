import { Injectable } from '@angular/core';
import { TauriBaseService } from '../core/tauri-base.service';
import { EventListenersService } from './event-listeners.service';
import { inject } from '@angular/core';
import { BehaviorSubject } from 'rxjs';

export interface RcloneUpdateInfo {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  current_version_clean: string;
  latest_version_clean: string;
  release_notes?: string;
  release_date?: string;
  download_url?: string;
}

export interface UpdateStatus {
  checking: boolean;
  updating: boolean;
  available: boolean;
  error: string | null;
  lastCheck: Date | null;
  updateInfo: RcloneUpdateInfo | null;
}

export interface UpdateResult {
  success: boolean;
  message?: string;
}

@Injectable({
  providedIn: 'root',
})
export class RcloneUpdateService extends TauriBaseService {
  private updateStatusSubject = new BehaviorSubject<UpdateStatus>({
    checking: false,
    updating: false,
    available: false,
    error: null,
    lastCheck: null,
    updateInfo: null,
  });

  public updateStatus$ = this.updateStatusSubject.asObservable();

  private eventListenersService = inject(EventListenersService);

  constructor() {
    super();
    this.setupEventListeners();
  }

  private setupEventListeners(): void {
    // Listen for engine update started
    this.eventListenersService.listenToEngineUpdateStarted().subscribe(() => {
      this.updateStatus({ updating: true });
    });

    // Listen for engine update completed
    this.eventListenersService.listenToEngineUpdateCompleted().subscribe(event => {
      this.updateStatus({
        updating: false,
        available: !event.payload.success,
      });
      if (event.payload.success) {
        this.checkForUpdates();
      }
    });

    // Listen for engine restarted
    this.eventListenersService.listenToEngineRestarted().subscribe(event => {
      if (event.payload.reason === 'rclone_update') {
        this.checkForUpdates();
      }
    });
  }

  async checkForUpdates(): Promise<RcloneUpdateInfo | null> {
    this.updateStatus({ checking: true, error: null });

    try {
      const updateInfo = await this.invokeCommand<RcloneUpdateInfo>('check_rclone_update');

      this.updateStatus({
        checking: false,
        available: updateInfo.update_available,
        lastCheck: new Date(),
        updateInfo: updateInfo,
      });

      return updateInfo;
    } catch (error) {
      console.error('Failed to check for updates:', error);
      this.updateStatus({
        checking: false,
        error: error as string,
        lastCheck: new Date(),
      });
      return null;
    }
  }

  async getDetailedUpdateInfo(): Promise<RcloneUpdateInfo> {
    try {
      return await this.invokeCommand<RcloneUpdateInfo>('get_rclone_update_info');
    } catch (error) {
      console.error('Failed to get detailed update info:', error);
      throw error;
    }
  }

  async performUpdate(): Promise<boolean> {
    this.updateStatus({ updating: true, error: null });

    try {
      const result = await this.invokeCommand<UpdateResult>('update_rclone');

      if (result.success) {
        this.updateStatus({
          updating: false,
          available: false,
          updateInfo: null,
        });

        // Log the successful update with path info if available
        if ('path' in result) {
          console.log('Rclone updated successfully at:', result.path);
        }

        return true;
      } else {
        this.updateStatus({
          updating: false,
          error: result.message || 'Update failed',
        });
        return false;
      }
    } catch (error) {
      console.error('Failed to update rclone:', error);
      this.updateStatus({
        updating: false,
        error: error as string,
      });
      return false;
    }
  }

  private updateStatus(update: Partial<UpdateStatus>): void {
    const currentStatus = this.updateStatusSubject.value;
    this.updateStatusSubject.next({ ...currentStatus, ...update });
  }

  getCurrentStatus(): UpdateStatus {
    return this.updateStatusSubject.value;
  }
}
