package dev.evvie.waylandcraft.bridge;

import dev.evvie.waylandcraft.render.WindowFramebuffer;

public abstract class WLCAbstractWindow {
	
	// Set to zero when this window no longer exists
	private long handle;
	
	protected WLCSurface surface;
	
	protected boolean wasMapped = false;
	
	public SurfaceGeometry geometry;
	
	public WLCAbstractWindow(long handle) {
		this.handle = handle;
	}
	
	public long getHandle() {
		return this.handle;
	}
	
	protected long takeHandle() {
		long old = this.handle;
		this.handle = 0;
		return old;
	}
	
	public boolean isAlive() {
		return handle != 0;
	}
	
	public WLCSurface getRootSurface() {
		return this.surface;
	}
	
	public WindowFramebuffer getFramebuffer() {
		return surface.getFramebuffer();
	}
	
	public boolean isMapped() {
		return isAlive() && getRootSurface().getBuffer() != null;
	}
	
	public static record SurfaceGeometry(int x, int y, int width, int height) {
	}
	
}
